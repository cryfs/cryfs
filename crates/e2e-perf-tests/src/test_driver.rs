use std::cell::RefCell;

use cryfs_blobstore::BlobStore as _;
use cryfs_rustfs::AtimeUpdateBehavior;
use pretty_assertions::assert_eq;

use crate::{
    filesystem_driver::FilesystemDriver,
    fixture::{ActionCounts, FilesystemFixture},
    rstest::{FixtureFactory, FixtureType},
};

#[must_use]
pub struct TestDriver<FS, FF>
where
    FS: FilesystemDriver,
    FF: FixtureFactory<Driver = FS>,
{
    fixture_factory: FF,
    atime_behavior: AtimeUpdateBehavior,
}

impl<FS, FF> TestDriver<FS, FF>
where
    FS: FilesystemDriver,
    FF: FixtureFactory<Driver = FS>,
{
    #[must_use]
    pub fn new(fixture_factory: FF, atime_behavior: AtimeUpdateBehavior) -> Self {
        Self {
            fixture_factory,
            atime_behavior,
        }
    }

    #[must_use]
    pub fn create_filesystem(
        self,
    ) -> TestDriverWithFs<FS, impl AsyncFn() -> FilesystemFixture<FS>> {
        let fixture_type = self.fixture_factory.fixture_type();
        TestDriverWithFs {
            filesystem: async move || {
                self.fixture_factory
                    .create_filesystem(self.atime_behavior)
                    .await
            },
            fixture_type,
        }
    }

    #[must_use]
    pub fn create_uninitialized_filesystem(
        self,
    ) -> TestDriverWithFs<FS, impl AsyncFn() -> FilesystemFixture<FS>> {
        let fixture_type = self.fixture_factory.fixture_type();
        TestDriverWithFs {
            filesystem: async move || {
                self.fixture_factory
                    .create_uninitialized_filesystem(self.atime_behavior)
                    .await
            },
            fixture_type,
        }
    }
}

#[must_use]
pub struct TestDriverWithFs<FS, CreateFsFn>
where
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<FS>,
{
    filesystem: CreateFsFn,
    fixture_type: FixtureType,
}

impl<FS, CreateFsFn> TestDriverWithFs<FS, CreateFsFn>
where
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<FS>,
{
    #[must_use]
    pub fn setup<SetupFn, SetupResult>(
        self,
        setup_fn: SetupFn,
    ) -> TestDriverWithFsAndSetupOp<
        FS,
        CreateFsFn,
        impl AsyncFn(&mut FilesystemFixture<FS>) -> SetupResult,
        SetupResult,
    >
    where
        SetupFn: AsyncFn(&mut FilesystemFixture<FS>) -> SetupResult,
    {
        TestDriverWithFsAndSetupOp {
            fixture_type: self.fixture_type,
            filesystem: self.filesystem,
            setup_fn: async move |fs| {
                let setup_result = setup_fn(fs).await;
                fs.blobstore.clear_cache_slow().await.unwrap();
                setup_result
            },
        }
    }

    #[must_use]
    pub fn setup_noflush<SetupFn, SetupResult>(
        self,
        setup_fn: SetupFn,
    ) -> TestDriverWithFsAndSetupOp<FS, CreateFsFn, SetupFn, SetupResult>
    where
        SetupFn: AsyncFn(&mut FilesystemFixture<FS>) -> SetupResult,
    {
        TestDriverWithFsAndSetupOp {
            fixture_type: self.fixture_type,
            filesystem: self.filesystem,
            setup_fn,
        }
    }
}

#[must_use]
pub struct TestDriverWithFsAndSetupOp<FS, CreateFsFn, SetupFn, SetupResult>
where
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<FS>) -> SetupResult,
{
    fixture_type: FixtureType,
    filesystem: CreateFsFn,
    setup_fn: SetupFn,
}

impl<FS, CreateFsFn, SetupFn, SetupResult>
    TestDriverWithFsAndSetupOp<FS, CreateFsFn, SetupFn, SetupResult>
where
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<FS>) -> SetupResult,
{
    #[must_use]
    pub fn test<TestFn>(
        self,
        test_fn: TestFn,
    ) -> TestDriverWithFsAndSetupOpAndTestOp<
        FS,
        CreateFsFn,
        SetupFn,
        SetupResult,
        impl AsyncFn(&mut FilesystemFixture<FS>, SetupResult),
    >
    where
        TestFn: AsyncFn(&mut FilesystemFixture<FS>, SetupResult),
    {
        self.test_noflush(async move |fs, setup_result| {
            let test_result = (test_fn)(fs, setup_result).await;
            fs.blobstore.clear_cache_slow().await.unwrap();
            test_result
        })
    }

    #[must_use]
    pub fn test_noflush<TestFn>(
        self,
        test_fn: TestFn,
    ) -> TestDriverWithFsAndSetupOpAndTestOp<FS, CreateFsFn, SetupFn, SetupResult, TestFn>
    where
        TestFn: AsyncFn(&mut FilesystemFixture<FS>, SetupResult),
    {
        TestDriverWithFsAndSetupOpAndTestOp {
            fixture_type: self.fixture_type,
            filesystem: self.filesystem,
            setup_fn: self.setup_fn,
            test_fn,
        }
    }
}

#[must_use]
pub struct TestDriverWithFsAndSetupOpAndTestOp<FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<FS>, SetupResult),
{
    fixture_type: FixtureType,
    filesystem: CreateFsFn,
    setup_fn: SetupFn,
    test_fn: TestFn,
}

impl<FS, CreateFsFn, SetupFn, SetupResult, TestFn>
    TestDriverWithFsAndSetupOpAndTestOp<FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<FS>, SetupResult),
{
    #[must_use]
    pub fn expect_op_counts(
        self,
        expected: impl FnOnce(FixtureType) -> ActionCounts,
    ) -> impl TestReady {
        TestReadyImpl {
            filesystem: self.filesystem,
            setup_fn: self.setup_fn,
            test_fn: self.test_fn,
            expected_op_counts: expected(self.fixture_type),
        }
    }
}

#[must_use]
pub trait TestReady {
    /// Run the test and assert that the operation counts match the expected values.
    fn assert_op_counts(&self);

    #[cfg(feature = "benchmark")]
    fn run_benchmark(&self, b: &mut criterion::Bencher);
}

#[must_use]
pub struct TestReadyImpl<FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<FS>, SetupResult),
{
    filesystem: CreateFsFn,
    setup_fn: SetupFn,
    test_fn: TestFn,
    expected_op_counts: ActionCounts,
}

impl<FS, CreateFsFn, SetupFn, SetupResult, TestFn>
    TestReadyImpl<FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<FS>, SetupResult),
{
    async fn _execute_test(&self) -> ActionCounts {
        let mut filesystem = (self.filesystem)().await;
        let setup_result = (self.setup_fn)(&mut filesystem).await;

        filesystem.blobstore.get_and_reset_counts();
        filesystem.hl_blockstore.get_and_reset_counts();
        filesystem.ll_blockstore.get_and_reset_counts();
        (self.test_fn)(&mut filesystem, setup_result).await;
        let counts = ActionCounts {
            blobstore: filesystem.blobstore.get_and_reset_counts(),
            high_level: filesystem.hl_blockstore.get_and_reset_counts(),
            low_level: filesystem.ll_blockstore.get_and_reset_counts(),
        };

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

impl<FS, CreateFsFn, SetupFn, SetupResult, TestFn> TestReady
    for TestReadyImpl<FS, CreateFsFn, SetupFn, SetupResult, TestFn>
where
    FS: FilesystemDriver,
    CreateFsFn: AsyncFn() -> FilesystemFixture<FS>,
    SetupFn: AsyncFn(&mut FilesystemFixture<FS>) -> SetupResult,
    TestFn: AsyncFn(&mut FilesystemFixture<FS>, SetupResult),
{
    fn assert_op_counts(&self) {
        let expected = self.expected_op_counts;
        let actual = Self::_new_runtime().block_on(async { self._execute_test().await });
        assert_eq!(expected, actual, "Action counts mismatch expectations");
    }

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
        let test_fn = |input: &mut RefCell<Option<(FilesystemFixture<FS>, SetupResult)>>| {
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
