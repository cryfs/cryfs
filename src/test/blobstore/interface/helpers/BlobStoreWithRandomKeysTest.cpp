#include <gtest/gtest.h>
#include <gmock/gmock.h>

#include "blobstore/interface/helpers/BlobStoreWithRandomKeys.h"
#include "blobstore/utils/RandomKeyGenerator.h"

using ::testing::Test;
using ::testing::_;
using ::testing::Return;
using ::testing::Invoke;

using std::string;
using std::unique_ptr;
using std::make_unique;

using namespace blobstore;

class BlobStoreWithRandomKeysMock: public BlobStoreWithRandomKeys {
public:
  unique_ptr<BlobWithKey> create(const std::string &key, size_t size) {
    return unique_ptr<BlobWithKey>(do_create(key, size));
  }
  MOCK_METHOD2(do_create, BlobWithKey*(const std::string &, size_t));
  unique_ptr<Blob> load(const string &key) {
    return unique_ptr<Blob>(do_load(key));
  }
  MOCK_METHOD1(do_load, Blob*(const string &));
  MOCK_METHOD1(exists, bool(const string &));
};

class BlobMock: public Blob {
public:
  MOCK_METHOD0(data, void*());
  MOCK_CONST_METHOD0(data, const void*());
  MOCK_METHOD0(flush, void());
  MOCK_CONST_METHOD0(size, size_t());
};

class BlobStoreWithRandomKeysTest: public Test {
public:
  BlobStoreWithRandomKeysMock blobStoreMock;
  BlobStore &blobStore = blobStoreMock;
};

TEST_F(BlobStoreWithRandomKeysTest, SizeIsPassedThrough0) {
  EXPECT_CALL(blobStoreMock, do_create(_, 0)).WillOnce(Return(new BlobWithKey("", make_unique<BlobMock>())));
  blobStore.create(0);
}

TEST_F(BlobStoreWithRandomKeysTest, SizeIsPassedThrough1) {
  EXPECT_CALL(blobStoreMock, do_create(_, 1)).WillOnce(Return(new BlobWithKey("", make_unique<BlobMock>())));
  blobStore.create(1);
}

TEST_F(BlobStoreWithRandomKeysTest, SizeIsPassedThrough1024) {
  EXPECT_CALL(blobStoreMock, do_create(_, 1024)).WillOnce(Return(new BlobWithKey("", make_unique<BlobMock>())));
  blobStore.create(1024);
}

TEST_F(BlobStoreWithRandomKeysTest, KeyHasCorrectSize) {
  EXPECT_CALL(blobStoreMock, do_create(_, _)).WillOnce(Invoke([](const string &key, size_t) {
    EXPECT_EQ(RandomKeyGenerator::KEYLENGTH, key.size());
    return new BlobWithKey("", make_unique<BlobMock>());
  }));

  blobStore.create(1024);
}

TEST_F(BlobStoreWithRandomKeysTest, TwoBlobsGetDifferentKeys) {
  string first_key;
  EXPECT_CALL(blobStoreMock, do_create(_, _))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        first_key = key;
        return new BlobWithKey("", make_unique<BlobMock>());
      }))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        EXPECT_NE(first_key, key);
        return new BlobWithKey("", make_unique<BlobMock>());
      }));

  blobStore.create(1024);
  blobStore.create(1024);
}

TEST_F(BlobStoreWithRandomKeysTest, WillTryADifferentKeyIfKeyAlreadyExists) {
  string first_key;
  EXPECT_CALL(blobStoreMock, do_create(_, _))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        first_key = key;
        return nullptr;
      }))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        EXPECT_NE(first_key, key);
        return new BlobWithKey("", make_unique<BlobMock>());
      }));

  blobStore.create(1024);
}

TEST_F(BlobStoreWithRandomKeysTest, WillTryADifferentKeyIfKeyAlreadyExistsTwoTimes) {
  string first_key;
  EXPECT_CALL(blobStoreMock, do_create(_, _))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        first_key = key;
        return nullptr;
      }))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        first_key = key;
        return nullptr;
      }))
      .WillOnce(Invoke([&first_key](const string &key, size_t) {
        EXPECT_NE(first_key, key);
        return new BlobWithKey("", make_unique<BlobMock>());
      }));

  blobStore.create(1024);
}
