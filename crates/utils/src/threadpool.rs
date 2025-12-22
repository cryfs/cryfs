use anyhow::Result;
use std::fmt::Debug;

/// A simple thread pool that can run CPU bound jobs and integrate them with async code.
pub struct ThreadPool {
    pool: rayon::ThreadPool,
}

impl ThreadPool {
    /// Create a new thread pool with a given name
    pub fn new(name: &'static str) -> Result<Self> {
        Self::new_with_num_threads(name, num_threads())
    }

    pub fn new_with_num_threads(name: &'static str, num_threads: usize) -> Result<Self> {
        Ok(Self {
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .thread_name(move |i| format!("{name} ({i})"))
                .build()?,
        })
    }

    /// Run a job on the thread pool and asynchronously wait for it to complete
    pub async fn execute_job<R>(&self, job: impl FnOnce() -> R + Send + 'static) -> R
    where
        R: Send + Debug + 'static,
    {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.pool.spawn_fifo(move || {
            let result = job();
            sender.send(result).unwrap();
        });
        receiver.await.expect("Thread pool task panicked")
    }
}

fn num_threads() -> usize {
    match std::thread::available_parallelism() {
        Ok(nz) => {
            let nz = nz.get();
            log::info!("Using parallelism factor of {nz}");
            nz
        }
        Err(err) => {
            log::warn!(
                "Could not determine number of cpu cores. Falling back to a parallelism factor of 2. Error: {err:?}"
            );
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn test_execute_simple_job() {
        let pool = ThreadPool::new("test-pool").unwrap();
        let result = pool.execute_job(|| 42).await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_execute_job_with_computation() {
        let pool = ThreadPool::new("test-pool").unwrap();
        let result = pool
            .execute_job(|| {
                let mut sum = 0;
                for i in 1..=100 {
                    sum += i;
                }
                sum
            })
            .await;
        assert_eq!(result, 5050);
    }

    #[tokio::test]
    async fn test_execute_multiple_jobs_sequentially() {
        let pool = ThreadPool::new("test-pool").unwrap();

        let result1 = pool.execute_job(|| 1 + 1).await;
        let result2 = pool.execute_job(|| 2 * 3).await;
        let result3 = pool.execute_job(|| 10 - 5).await;

        assert_eq!(result1, 2);
        assert_eq!(result2, 6);
        assert_eq!(result3, 5);
    }

    #[tokio::test]
    async fn test_execute_multiple_jobs_concurrently() {
        let pool = ThreadPool::new("test-pool").unwrap();
        let counter = Arc::new(AtomicU32::new(0));

        let mut futures = vec![];
        for _ in 0..10 {
            let counter_clone = Arc::clone(&counter);
            let future = pool.execute_job(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
            });
            futures.push(future);
        }

        futures::future::join_all(futures).await;

        assert_eq!(counter.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn test_execute_job_with_string_return() {
        let pool = ThreadPool::new("test-pool").unwrap();
        let result = pool.execute_job(|| String::from("hello world")).await;
        assert_eq!(result, "hello world");
    }

    #[tokio::test]
    async fn test_execute_job_with_complex_type() {
        let pool = ThreadPool::new("test-pool").unwrap();
        let result = pool
            .execute_job(|| {
                vec![1, 2, 3, 4, 5]
                    .into_iter()
                    .map(|x| x * 2)
                    .collect::<Vec<_>>()
            })
            .await;
        assert_eq!(result, vec![2, 4, 6, 8, 10]);
    }

    #[tokio::test]
    async fn test_execute_job_captures_environment() {
        let pool = ThreadPool::new("test-pool").unwrap();
        let value = 100;
        let result = pool.execute_job(move || value * 2).await;
        assert_eq!(result, 200);
    }

    #[tokio::test]
    async fn test_execute_blocking_operation() {
        let pool = ThreadPool::new("test-pool").unwrap();
        let start = std::time::Instant::now();

        let result = pool
            .execute_job(|| {
                std::thread::sleep(Duration::from_millis(100));
                "completed"
            })
            .await;

        let elapsed = start.elapsed();
        assert_eq!(result, "completed");
        assert!(elapsed >= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_thread_pool_reuse() {
        let pool = ThreadPool::new("test-pool").unwrap();

        for i in 0..20 {
            let result = pool.execute_job(move || i * 2).await;
            assert_eq!(result, i * 2);
        }
    }

    #[tokio::test]
    async fn test_concurrent_jobs_with_shared_state() {
        let pool = ThreadPool::new("test-pool").unwrap();
        let counter = Arc::new(AtomicU32::new(0));

        let mut futures = vec![];
        for i in 0..100 {
            let counter_clone = Arc::clone(&counter);
            let future = pool.execute_job(move || {
                std::thread::sleep(Duration::from_micros(100));
                counter_clone.fetch_add(i, Ordering::SeqCst)
            });
            futures.push(future);
        }

        futures::future::join_all(futures).await;

        let expected_sum = (0..100).sum();
        assert_eq!(counter.load(Ordering::SeqCst), expected_sum);
    }

    #[test]
    fn test_num_threads() {
        let threads = num_threads();
        assert!(threads >= 1, "Should have at least 1 thread");
    }

    #[tokio::test]
    async fn test_new_thread_pool_creation() {
        let result = ThreadPool::new("test-pool");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_jobs_execute_in_parallel() {
        // Create a pool with at least 2 threads
        let pool = ThreadPool::new_with_num_threads("test-pool", 2).unwrap();

        // Create a barrier that requires 2 threads to reach it
        let barrier = Arc::new(std::sync::Barrier::new(2));

        // Launch 2 jobs that will both wait at the barrier
        let barrier1 = Arc::clone(&barrier);
        let future1 = pool.execute_job(move || {
            // Wait at the barrier - this will block until both jobs reach this point
            barrier1.wait();
            1
        });

        let barrier2 = Arc::clone(&barrier);
        let future2 = pool.execute_job(move || {
            // Wait at the barrier - this will block until both jobs reach this point
            barrier2.wait();
            2
        });

        // If jobs execute sequentially, the first would block forever waiting for the second
        // If jobs execute in parallel, both will reach the barrier and unblock each other
        let (result1, result2) = tokio::join!(future1, future2);
        assert_eq!(result1, 1);
        assert_eq!(result2, 2);
    }

    /// Test that the threadpool works correctly when called from futures::executor::block_on
    /// This is important because some code paths (like InodeGuardInner::drop) use futures::executor
    /// instead of tokio.
    #[test]
    fn test_execute_job_from_futures_executor() {
        let pool = ThreadPool::new("test-pool").unwrap();

        // Simulate the problematic scenario: calling execute_job from within futures::executor::block_on
        let result = futures::executor::block_on(async { pool.execute_job(|| 42).await });

        assert_eq!(result, 42);
    }

    /// Test multiple sequential calls from futures::executor::block_on
    #[test]
    fn test_multiple_jobs_from_futures_executor() {
        let pool = ThreadPool::new("test-pool").unwrap();

        futures::executor::block_on(async {
            let r1 = pool.execute_job(|| 1).await;
            let r2 = pool.execute_job(|| 2).await;
            let r3 = pool.execute_job(|| 3).await;
            assert_eq!(r1 + r2 + r3, 6);
        });
    }
}
