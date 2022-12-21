use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use tokio::{
    select,
    task::{self, JoinHandle},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

use crate::async_drop::{AsyncDrop, AsyncDropGuard};

/// Implements a task that runs periodically with a given interval,
/// as long as the instance is alive, and will stop executing when
/// the [PeriodicTask] instance is destructed or when [PeriodicTask::terminate]
/// is called.
///
/// Termination will never interrupt an actively running task iteration.
/// If it is terminated while running, then the current run will complete
/// and only future executions will be cancelled. If termination happens
/// through [Drop], then the current thread gets blocked until termination
/// is complete.
///
/// Warning:
/// --------
/// This only works on a multi-thread runtime, otherwise [Drop] will cause a deadlock
/// because it'll call [tokio::runtime::Handle::block_on].
pub struct PeriodicTask {
    task_impl: Arc<dyn PeriodicTaskImplTerminate>,
    // None if it was already joined
    join_handle: Option<JoinHandle<()>>,
}

impl PeriodicTask {
    /// Spawn a new periodic task that runs in the background.
    /// Every `interval`, `task` will be run asynchronously on the
    /// current tokio runtime.
    pub fn spawn<T, F>(name: &'static str, interval: Duration, task: T) -> AsyncDropGuard<Self>
    where
        F: Send + Future<Output = Result<()>> + 'static,
        // TODO This could probably be FnMut since we're the only ones accessing it.
        //      Think it through and if it is true, make it so.
        T: Send + Sync + 'static + Fn() -> F,
    {
        let task_impl = Arc::new(PeriodicTaskImpl {
            name,
            task,
            interval,
            should_terminate: CancellationToken::new(),
        });
        let join_handle = Arc::clone(&task_impl)._run();
        AsyncDropGuard::new(Self {
            task_impl,
            join_handle: Some(join_handle),
        })
    }

    /// Terminate the task and await until it is terminated.
    /// If the task is currently actively executing, that execution
    /// will first finish and then the task will be interrupted,
    /// i.e. there will be no future execution of it issued.
    pub async fn terminate(&mut self) -> Result<()> {
        self.task_impl.set_for_termination();
        if let Some(join_handle) = self.join_handle.take() {
            join_handle.await?;
        }
        Ok(())
    }
}

// TODO Use Drop instead of AsyncDrop? https://stackoverflow.com/questions/71541765/rust-async-drop
#[async_trait]
impl AsyncDrop for PeriodicTask {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<()> {
        self.terminate().await?;
        Ok(())
    }
}

impl Debug for PeriodicTask {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("PeriodicTask")
            .field("name", &self.task_impl.name())
            .finish()
    }
}

trait PeriodicTaskImplTerminate: Send + Sync {
    fn set_for_termination(&self);
    fn name(&self) -> &'static str;
}

struct PeriodicTaskImpl<T, F>
where
    F: Send + Future<Output = Result<()>> + 'static,
    T: Send + Sync + 'static + Fn() -> F,
{
    name: &'static str,
    task: T,
    interval: Duration,
    should_terminate: CancellationToken,
}

impl<T, F> PeriodicTaskImplTerminate for PeriodicTaskImpl<T, F>
where
    F: Send + Future<Output = Result<()>> + 'static,
    T: Send + Sync + 'static + Fn() -> F,
{
    fn set_for_termination(&self) {
        self.should_terminate.cancel();
    }

    fn name(&self) -> &'static str {
        &self.name
    }
}

impl<T, F> PeriodicTaskImpl<T, F>
where
    F: Send + Future<Output = Result<()>> + 'static,
    T: Send + Sync + 'static + Fn() -> F,
{
    fn _run(self: Arc<Self>) -> JoinHandle<()> {
        let cloned_token = self.should_terminate.clone();
        task::spawn(async move {
            loop {
                select! {
                    _ = cloned_token.cancelled() => {
                        break;
                    }
                    _ = tokio::time::sleep(self.interval) => {
                        let result = (self.task)().await;
                        match result {
                            Ok(()) =>
                            /* do nothing, continue loop */
                            {
                                ()
                            }
                            // TODO What should we do on error? Just log? Or panic?
                            Err(err) => log::error!("Error in periodic task {}: {:?}", self.name, err),
                        }
                    }
                };
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PeriodicTask;
    use anyhow::bail;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };
    use tokio::time::Duration;

    // TODO Some way to test that the interval has the correct length?

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn run_empty_task() {
        let mut task =
            PeriodicTask::spawn("Test Task", Duration::from_millis(1), || async { Ok(()) });
        task.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn runs_several_times() {
        let was_run = Arc::new(AtomicU32::new(0));
        let was_run_clone = Arc::clone(&was_run);
        let mut task = PeriodicTask::spawn("Test Task", Duration::from_millis(1), move || {
            let was_run_clone = Arc::clone(&was_run_clone);
            async move {
                // Add an additional sleep to give control back to the event loop
                // and make sure that it actually calls back into our future
                tokio::time::sleep(Duration::from_millis(1)).await;
                was_run_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
        while was_run.load(Ordering::SeqCst) < 10 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        task.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn terminate_stops_execution() {
        let was_run = Arc::new(AtomicU32::new(0));
        let was_run_clone = Arc::clone(&was_run);
        let mut task = PeriodicTask::spawn("Test Task", Duration::from_millis(1), move || {
            let was_run_clone = Arc::clone(&was_run_clone);
            async move {
                // Add an additional sleep to give control back to the event loop
                // and make sure that it actually calls back into our future
                tokio::time::sleep(Duration::from_millis(1)).await;
                was_run_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
        // Run a couple of times
        while was_run.load(Ordering::SeqCst) < 10 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        task.terminate().await.unwrap();

        // Make sure it doesn't run anymore
        let num_runs = was_run.load(Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(num_runs, was_run.load(Ordering::SeqCst));

        task.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn drop_stops_execution() {
        let was_run = Arc::new(AtomicU32::new(0));
        let was_run_clone = Arc::clone(&was_run);
        let mut task = PeriodicTask::spawn("Test Task", Duration::from_millis(1), move || {
            let was_run_clone = Arc::clone(&was_run_clone);
            async move {
                // Add an additional sleep to give control back to the event loop
                // and make sure that it actually calls back into our future
                tokio::time::sleep(Duration::from_millis(1)).await;
                was_run_clone.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        });
        // Run a couple of times
        while was_run.load(Ordering::SeqCst) < 10 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }

        task.async_drop().await.unwrap();

        // Make sure it doesn't run anymore
        let num_runs = was_run.load(Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(num_runs, was_run.load(Ordering::SeqCst));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn run_panic_task() {
        // TODO Why isn't this panic reported back?
        let mut task = PeriodicTask::spawn("Test Task", Duration::from_millis(1), || async {
            panic!("my error")
        });
        task.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn run_error_task() {
        // TODO Why isn't this error reported back?
        let mut task = PeriodicTask::spawn("Test Task", Duration::from_millis(1), || async {
            bail!("my error")
        });
        task.async_drop().await.unwrap();
    }
}
