use anyhow::Result;
use futures::future::{BoxFuture, FutureExt};
use futures::stream::{StreamExt, TryStreamExt};
use std::future::Future;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Call this with an `initial_task` and it will call that task, and all tasks recursively spawned by it, until all tasks are done.
/// It will restrict the number of tasks running concurrently to `max_concurrency`.
pub async fn run_to_completion<'f, F>(
    max_concurrency: usize,
    initial_task: impl FnOnce(TaskSpawner<'f>) -> F,
) -> Result<()>
where
    F: Future<Output = Result<()>> + Send + 'f,
{
    let (sender, receiver) = unbounded_channel();
    TaskSpawner { sender }.spawn(initial_task);

    UnboundedReceiverStream::new(receiver)
        .buffer_unordered(max_concurrency)
        .try_collect()
        .await?;
    Ok(())
}

pub struct TaskSpawner<'f> {
    sender: UnboundedSender<BoxFuture<'f, Result<()>>>,
}

impl<'f> TaskSpawner<'f> {
    pub fn spawn<F>(&self, future: impl FnOnce(Self) -> F)
    where
        F: Future<Output = Result<()>> + Send + 'f,
    {
        let future = future(TaskSpawner {
            sender: self.sender.clone(),
        });
        let future = future.boxed();
        self.sender.send(future).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn spawn_100_tasks_directly() {
        use super::*;
        use std::sync::atomic::{AtomicUsize, Ordering};
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        run_to_completion(10, |spawner| async move {
            for _ in 0..100 {
                let counter_clone = Arc::clone(&counter_clone);
                spawner.spawn(move |_| async move {
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                });
            }
            Ok(())
        })
        .await
        .unwrap();
        assert_eq!(100, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn spawn_100_tasks_recursively() {
        use super::*;
        use std::sync::atomic::{AtomicUsize, Ordering};
        let counter = Arc::new(AtomicUsize::new(0));

        #[async_recursion::async_recursion]
        async fn task(
            spawner: TaskSpawner<'static>,
            counter: Arc<AtomicUsize>,
            index: usize,
        ) -> Result<()> {
            counter.fetch_add(1, Ordering::SeqCst);
            if index < 100 {
                spawner
                    .spawn(move |spawner| async move { task(spawner, counter, index + 1).await });
            }
            Ok(())
        }

        run_to_completion(10, |spawner| task(spawner, Arc::clone(&counter), 1))
            .await
            .unwrap();
        assert_eq!(100, counter.load(Ordering::SeqCst));
    }
}
