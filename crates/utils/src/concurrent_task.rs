use std::mem::ManuallyDrop;

use tokio::task::{JoinError, JoinHandle};

/// A [ConcurrentTask] is a task that can be spawned and then later awaited on.
/// As opposed to tokio tasks, a [ConcurrentTask] must be awaited at some point.
/// Not awaiting it will be a compile time error.
pub struct ConcurrentTask<T>
where
    T: Send + 'static,
{
    task: ManuallyDrop<JoinHandle<T>>,
}

impl<T> ConcurrentTask<T>
where
    T: Send + 'static,
{
    /// Spawn a new task concurrently in the background.
    pub fn spawn(task: impl Future<Output = T> + Send + 'static) -> Self {
        Self {
            task: ManuallyDrop::new(tokio::spawn(task)),
        }
    }

    /// Await the task and return the result.
    /// This method must be called exactly once and it is a compiler error to not call it.
    #[must_use]
    pub fn await_task(self) -> impl Future<Output = Result<T, JoinError>> {
        let mut this = ManuallyDrop::new(self);
        
        unsafe { ManuallyDrop::take(&mut this.task) }
    }
}

impl<T> Drop for ConcurrentTask<T>
where
    T: Send + 'static,
{
    fn drop(&mut self) {
        const {
            panic!("Forgot to call ConcurrentTask::await_task()");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_task() {
        let task = ConcurrentTask::spawn(async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            5
        });

        let v = task.await_task().await.unwrap();
        assert_eq!(5, v);
    }
}
