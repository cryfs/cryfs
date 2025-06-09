#[cfg(not(feature = "benchmark"))]
use pretty_assertions::assert_eq;
#[cfg(feature = "benchmark")]
use std::cell::RefCell;
use std::marker::PhantomData;

use crate::{
    filesystem_driver::FilesystemDriver,
    filesystem_fixture::{ActionCounts, FilesystemFixture},
    perf_test_macro::FixtureType,
};
use cryfs_blobstore::BlobStore as _;
use cryfs_blockstore::{LLBlockStore, OptimizedBlockStoreWriter};
use cryfs_rustfs::AtimeUpdateBehavior;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

/// A [TestDriver] offers a builder API for defining a test case that can then
/// be instantiated with the [crate::perf_test_macro::perf_test!] macro. Using this builder API,
/// the test case defines the setup code and operations it wants to run,
/// and [crate::perf_test_macro::perf_test!] will then make that into counter tests or benchmarks.
#[must_use]
pub trait TestDriver {
    type B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync;
    type FS: FilesystemDriver;

    /// See [FilesystemFixture::create_filesystem] for details.
    #[must_use]
    fn create_filesystem(
        self,
    ) -> TestDriverWithFs<Self::B, Self::FS, impl AsyncFn() -> FilesystemFixture<Self::B, Self::FS>>;

    /// See [FilesystemFixture::create_uninitialized_filesystem] for details.
    #[must_use]
    fn create_uninitialized_filesystem(
        self,
    ) -> TestDriverWithFs<Self::B, Self::FS, impl AsyncFn() -> FilesystemFixture<Self::B, Self::FS>>;
}

#[must_use]
pub struct TestDriverImpl<B, CreateBlockstoreFn, FS>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    CreateBlockstoreFn: Fn() -> AsyncDropGuard<B>,
    FS: FilesystemDriver,
{
    blockstore: CreateBlockstoreFn,
    atime_update_behavior: AtimeUpdateBehavior,
    _fsdriver: PhantomData<FS>,
    #[cfg(not(feature = "benchmark"))]
    fixture_type: FixtureType,
}

impl<B, CreateBlockstoreFn, FS> TestDriverImpl<B, CreateBlockstoreFn, FS>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    CreateBlockstoreFn: Fn() -> AsyncDropGuard<B>,
    FS: FilesystemDriver,
{
    #[must_use]
    pub fn new(
        blockstore: CreateBlockstoreFn,
        _fsdriver: PhantomData<FS>,
        #[allow(unused_variables)] fixture_type: FixtureType,
        atime_update_behavior: AtimeUpdateBehavior,
    ) -> Self {
        Self {
            blockstore,
            atime_update_behavior,
            _fsdriver: PhantomData,
            #[cfg(not(feature = "benchmark"))]
            fixture_type,
        }
    }
}

impl<B, CreateBlockstoreFn, FS> TestDriver for TestDriverImpl<B, CreateBlockstoreFn, FS>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    CreateBlockstoreFn: Fn() -> AsyncDropGuard<B>,
    FS: FilesystemDriver,
{
    type B = B;
    type FS = FS;

    /// See [TestDriver::create_filesystem]
    fn create_filesystem(
        self,
    ) -> TestDriverWithFs<B, FS, impl AsyncFn() -> FilesystemFixture<B, FS>> {
        #[cfg(not(feature = "benchmark"))]
        let fixture_type = self.fixture_type;
        let atime_update_behavior = self.atime_update_behavior;
        TestDriverWithFs {
            filesystem: async move || {
                FilesystemFixture::<B, FS>::create_filesystem(
                    (self.blockstore)(),
                    atime_update_behavior,
                )
                .await
            },
            #[cfg(not(feature = "benchmark"))]
            fixture_type,
            #[cfg(not(feature = "benchmark"))]
            atime_update_behavior,
        }
    }

    /// See [TestDriver::create_uninitialized_filesystem]
    fn create_uninitialized_filesystem(
        self,
    ) -> TestDriverWithFs<B, FS, impl AsyncFn() -> FilesystemFixture<B, FS>> {
        #[cfg(not(feature = "benchmark"))]
        let fixture_type = self.fixture_type;
        #[cfg(not(feature = "benchmark"))]
        let atime_update_behavior = self.atime_update_behavior;
        TestDriverWithFs {
            filesystem: async move || {
                FilesystemFixture::<B, FS>::create_uninitialized_filesystem(
                    (self.blockstore)(),
                    self.atime_update_behavior,
                )
                .await
            },
            #[cfg(not(feature = "benchmark"))]
            fixture_type,
            #[cfg(not(feature = "benchmark"))]
            atime_update_behavior,
        }
    }
}

#[must_use]
pub struct TestDriverWithFs<B, FS, CreateFsFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
{
    filesystem: CreateFsFn,
    #[cfg(not(feature = "benchmark"))]
    fixture_type: FixtureType,
    #[cfg(not(feature = "benchmark"))]
    atime_update_behavior: AtimeUpdateBehavior,
}

impl<B, FS, CreateFsFn> TestDriverWithFs<B, FS, CreateFsFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
{
    /// Add some setup code that will be executed before each test.
    /// Operatons run in this setup code will not influence the benchmark time,
    /// and if run as counter tests, counters will be reset to zero after the setup code.
    ///
    /// CryFS cache will be flushed after the setup code runs, meaning the operation starts with an empty cache.
    /// Use [Self::setup_noflush] if you don't want that.
    #[must_use]
    pub fn setup<SetupFn, SetupResult>(
        self,
        setup_fn: SetupFn,
    ) -> TestDriverWithFsAndSetupOp<
        B,
        FS,
        CreateFsFn,
        impl AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
        SetupResult,
    >
    where
        SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    {
        self.setup_noflush(async move |fs| {
            let setup_result = setup_fn(fs).await;
            fs.blobstore.clear_cache_slow().await.unwrap();
            setup_result
        })
    }

    /// Same as [Self::setup], but don't flush  the CryFS cache after the setup code finishes.
    /// This allows testing situations that don't start with an empty cache.
    #[must_use]
    pub fn setup_noflush<SetupFn, SetupResult>(
        self,
        setup_fn: SetupFn,
    ) -> TestDriverWithFsAndSetupOp<B, FS, CreateFsFn, SetupFn, SetupResult>
    where
        SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    {
        TestDriverWithFsAndSetupOp {
            #[cfg(not(feature = "benchmark"))]
            fixture_type: self.fixture_type,
            #[cfg(not(feature = "benchmark"))]
            atime_update_behavior: self.atime_update_behavior,
            filesystem: self.filesystem,
            setup_fn,
        }
    }
}

#[must_use]
pub struct TestDriverWithFsAndSetupOp<B, FS, CreateFsFn, SetupFn, SetupResult>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
{
    #[cfg(not(feature = "benchmark"))]
    fixture_type: FixtureType,
    #[cfg(not(feature = "benchmark"))]
    atime_update_behavior: AtimeUpdateBehavior,
    filesystem: CreateFsFn,
    setup_fn: SetupFn,
}

impl<B, FS, CreateFsFn, SetupFn, SetupResult>
    TestDriverWithFsAndSetupOp<B, FS, CreateFsFn, SetupFn, SetupResult>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
{
    /// See [TestDriverWithFs::setup_noflush]
    #[must_use]
    pub fn setup_noflush<SetupFn2, SetupResult2>(
        self,
        setup_fn_2: SetupFn2,
    ) -> TestDriverWithFsAndSetupOp<
        B,
        FS,
        CreateFsFn,
        impl AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult2,
        SetupResult2,
    >
    where
        SetupFn2: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult) -> SetupResult2,
    {
        TestDriverWithFsAndSetupOp {
            #[cfg(not(feature = "benchmark"))]
            fixture_type: self.fixture_type,
            #[cfg(not(feature = "benchmark"))]
            atime_update_behavior: self.atime_update_behavior,
            filesystem: self.filesystem,
            setup_fn: async move |fixture| {
                let setup_result = (self.setup_fn)(fixture).await;
                setup_fn_2(fixture, setup_result).await
            },
        }
    }

    /// Add a test operation. This is the code that will be counted for counter tests
    /// and that will run under timers for benchmark tests.
    ///
    /// By default, after the test code finishes, CryFS caches will be cleared, also within
    /// the counted section. This ensures that the operation doesn't become trivial because it just
    /// makes the cache dirty. Use [Self::test_noflush] if you don't want that.
    #[must_use]
    pub fn test<TestFn>(
        self,
        test_fn: TestFn,
    ) -> TestDriverWithFsAndSetupOpAndTestOp<
        B,
        FS,
        CreateFsFn,
        SetupFn,
        SetupResult,
        impl AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
    >
    where
        TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
    {
        self.test_noflush(async move |fs, setup_result| {
            let test_result = (test_fn)(fs, setup_result).await;
            fs.blobstore.clear_cache_slow().await.unwrap();
            test_result
        })
    }

    /// Same as [Self::test], but don't reset the operation counters after setup before executing the test code.
    /// This allows testing scenarios where we want to keep operation counts from the setup code and add counts from the test code to it.
    #[must_use]
    pub fn test_no_counter_reset<TestFn>(
        self,
        test_fn: TestFn,
    ) -> TestDriverWithFsAndSetupOpAndTestOp<
        B,
        FS,
        CreateFsFn,
        SetupFn,
        SetupResult,
        impl AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
    >
    where
        TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
    {
        self.test_noflush_no_counter_reset(async move |fs, setup_result| {
            let test_result = (test_fn)(fs, setup_result).await;
            fs.blobstore.clear_cache_slow().await.unwrap();
            test_result
        })
    }

    /// Same as [Self::test], but don't flush the CryFS cache after the test code finishes.
    /// This allows testing code that puts things into a cache, without forcing the cache flush.
    #[must_use]
    pub fn test_noflush<TestFn>(
        self,
        test_fn: TestFn,
    ) -> TestDriverWithFsAndSetupOpAndTestOp<
        B,
        FS,
        CreateFsFn,
        SetupFn,
        SetupResult,
        impl AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
    >
    where
        TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
    {
        self.test_noflush_no_counter_reset(
            async move |fs: &mut FilesystemFixture<B, FS>, setup_result: SetupResult| {
                fs.reset_counts();
                (test_fn)(fs, setup_result).await
            },
        )
    }

    /// See [Self::test_noflush], and [Self::test_no_counter_reset]. This function combines both.
    #[must_use]
    pub fn test_noflush_no_counter_reset<TestFn>(
        self,
        test_fn: TestFn,
    ) -> TestDriverWithFsAndSetupOpAndTestOp<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
    where
        TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
    {
        TestDriverWithFsAndSetupOpAndTestOp {
            #[cfg(not(feature = "benchmark"))]
            fixture_type: self.fixture_type,
            #[cfg(not(feature = "benchmark"))]
            atime_update_behavior: self.atime_update_behavior,
            filesystem: self.filesystem,
            setup_fn: self.setup_fn,
            test_fn,
        }
    }
}

#[must_use]
pub struct TestDriverWithFsAndSetupOpAndTestOp<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
{
    #[cfg(not(feature = "benchmark"))]
    fixture_type: FixtureType,
    #[cfg(not(feature = "benchmark"))]
    atime_update_behavior: AtimeUpdateBehavior,
    filesystem: CreateFsFn,
    setup_fn: SetupFn,
    test_fn: TestFn,
}

impl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
    TestDriverWithFsAndSetupOpAndTestOp<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
{
    /// Define the expected operation counts for performance tests.
    /// These are the numbers the actually run operations will be compared against in the test assertion.
    #[must_use]
    pub fn expect_op_counts(
        self,
        #[allow(unused_variables)] expected: impl FnOnce(
            FixtureType,
            AtimeUpdateBehavior,
        ) -> ActionCounts,
    ) -> impl TestReady {
        TestReadyImpl {
            filesystem: self.filesystem,
            setup_fn: self.setup_fn,
            test_fn: self.test_fn,
            #[cfg(not(feature = "benchmark"))]
            expected_op_counts: expected(self.fixture_type, self.atime_update_behavior),
        }
    }
}

#[must_use]
pub trait TestReady {
    /// Run as a performance counter test and assert that the operation counts match the expected values.
    #[cfg(not(feature = "benchmark"))]
    fn assert_op_counts(&self);

    /// Run as a benchmark with criterion
    #[cfg(feature = "benchmark")]
    fn run_benchmark(&self, b: &mut criterion::Bencher);
}

#[must_use]
pub struct TestReadyImpl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
{
    filesystem: CreateFsFn,
    setup_fn: SetupFn,
    test_fn: TestFn,
    #[cfg(not(feature = "benchmark"))]
    expected_op_counts: ActionCounts,
}

impl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
    TestReadyImpl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
{
    #[cfg(not(feature = "benchmark"))]
    async fn _execute_test(&self) -> ActionCounts {
        let mut filesystem = (self.filesystem)().await;
        let setup_result = (self.setup_fn)(&mut filesystem).await;

        (self.test_fn)(&mut filesystem, setup_result).await;
        let counts = filesystem.totals();

        counts
    }

    fn _new_runtime() -> tokio::runtime::Runtime {
        // Runtime setup copied from #[tokio::test] code
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    }
}

impl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn> TestReady
    for TestReadyImpl<B, FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    B: LLBlockStore + OptimizedBlockStoreWriter + AsyncDrop + Send + Sync,
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<B, FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<B, FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<B, FS>, SetupResult),
{
    /// See [TestReady::assert_op_counts]
    #[cfg(not(feature = "benchmark"))]
    fn assert_op_counts(&self) {
        let expected = self.expected_op_counts;
        let actual = Self::_new_runtime().block_on(async { self._execute_test().await });
        assert_eq!(expected, actual, "Action counts mismatch expectations");
    }

    /// See [TestReady::run_benchmark]
    #[cfg(feature = "benchmark")]
    fn run_benchmark(&self, b: &mut criterion::Bencher) {
        let setup_fn = || {
            // Need to use futures instead of tokio because tokio can't start a runtime inside a runtime and criterion calls this from within a runtime (but doesn't allow us to pass in an async function).
            futures::executor::block_on(async {
                let mut filesystem = (self.filesystem)().await;
                let setup_result = (self.setup_fn)(&mut filesystem).await;
                RefCell::new(Some((filesystem, setup_result)))
            })
        };
        let test_fn = |input: &mut RefCell<Option<(FilesystemFixture<B, FS>, SetupResult)>>| {
            let (mut filesystem, setup_result) = input.replace(None).expect(
                // The RefCell is a hack to make this FnMut (required by the criterion API) but we actually have a FnOnce.
                "Tried to run benchmark function multiple times without re-running setup",
            );
            async move {
                (self.test_fn)(&mut filesystem, setup_result).await;
            }
        };
        b.to_async(Self::_new_runtime()).iter_batched_ref(
            setup_fn,
            test_fn,
            criterion::BatchSize::PerIteration,
        );
    }
}
