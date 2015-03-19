#ifndef MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_H_
#define MESSMER_FSPP_FSTEST_FSPPOPENFILETEST_H_

template<class ConcreteFileSystemTestFixture>
class FsppOpenFileTest: public FileSystemTest<ConcreteFileSystemTestFixture> {
public:
};

TYPED_TEST_CASE_P(FsppOpenFileTest);

TYPED_TEST_P(FsppOpenFileTest, Bla) {
  //TODO
}

REGISTER_TYPED_TEST_CASE_P(FsppOpenFileTest,
  Bla
);

#endif
