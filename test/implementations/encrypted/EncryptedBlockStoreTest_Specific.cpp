#include <google/gtest/gtest.h>
#include "testutils/FakeAuthenticatedCipher.h"
#include "../../../implementations/encrypted/EncryptedBlockStore.h"
#include "../../../implementations/testfake/FakeBlockStore.h"

using ::testing::Test;

using std::unique_ptr;
using std::make_unique;

using blockstore::testfake::FakeBlockStore;

using namespace blockstore::encrypted;

class EncryptedBlockStoreTest: public Test {
public:
  EncryptedBlockStoreTest(): blockStore(make_unique<EncryptedBlockStore<FakeAuthenticatedCipher>>(make_unique<FakeBlockStore>(), FakeAuthenticatedCipher::Key1())) {}
  unique_ptr<EncryptedBlockStore<FakeAuthenticatedCipher>> blockStore;
};

TEST_F(EncryptedBlockStoreTest, LoadingWithSameKeyWorks) {
  //TODO implement
}

TEST_F(EncryptedBlockStoreTest, LoadingWithDifferentKeyDoesntWork) {
  //TODO Implement
}

TEST_F(EncryptedBlockStoreTest, LoadingModifiedBlockFails) {
  //TODO Implement
}

TEST_F(EncryptedBlockStoreTest, LoadingWithDifferentBlockIdFails) {
  //TODO loading it with a different blockstore::Key will fail (because it stores its key in a header)
}
