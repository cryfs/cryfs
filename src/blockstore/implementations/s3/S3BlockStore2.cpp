#include "S3BlockStore2.h"
#include <memory>
#include <cpp-utils/assert/assert.h>
#include <aws/core/Aws.h>
#include <aws/s3/S3Client.h>
#include <aws/s3/model/PutObjectRequest.h>
#include <aws/s3/model/GetObjectRequest.h>
#include <aws/s3/model/ListObjectsRequest.h>
#include <aws/s3/model/DeleteObjectRequest.h>
#include <boost/interprocess/streams/bufferstream.hpp>

using std::string;
using std::mutex;
using std::make_pair;
using std::vector;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using boost::optional;
using boost::none;

namespace blockstore {

// TODO Allow using a prefix directory inside a bucket

// TODO Make configurable
const char* region = "us-west-2";
const char* bucket_name = "test.cryfs";

namespace s3 {
namespace {
struct S3APIInitializer final {
    Aws::SDKOptions options;

    S3APIInitializer(): options() {
        Aws::InitAPI(options);
    }

    ~S3APIInitializer() {
        Aws::ShutdownAPI(options);
    }
};
}

struct S3BlockStore2::AwsS3SDK final {
public:
    AwsS3SDK() : _initializer(), _config(_clientConfig()), _client(*_config) {
    }

    bool put(const BlockId& blockId, const Data& data, bool allowOverwrite) {
        // TODO if !allowOverwrite and already exists, return false

        Aws::S3::Model::PutObjectRequest request;
        request.WithBucket(bucket_name).WithKey(blockId.ToString());
        Data copy = data.copy(); // copy so we can get a non-const char* for it
        // TODO request.WithContentLength(data.size()) ?
        request.SetBody(std::make_shared<boost::interprocess::bufferstream>(static_cast<char*>(copy.data()), copy.size()));

        auto response = _client.PutObject(request);
        if (!response.IsSuccess()) {
            throw std::runtime_error("AWS exception in PutObject: " + response.GetError().GetExceptionName() + ": " + response.GetError().GetMessage());
        }
        return true;
    }

    bool remove(const BlockId &blockId) {
        Aws::S3::Model::DeleteObjectRequest request;
        request.WithBucket(bucket_name).WithKey(blockId.ToString());

        auto response = _client.DeleteObject(request);

        if (!response.IsSuccess()) {
            // TODO return false if it just didn't exist
            throw std::runtime_error("AWS exception in DeleteObject: " + response.GetError().GetExceptionName() + ": " + response.GetError().GetMessage());
        }

        return true;
    }

    optional<Data> load(const BlockId &blockId) const {
        Aws::S3::Model::GetObjectRequest request;
        request.WithBucket(bucket_name).WithKey(blockId.ToString());

        auto response = _client.GetObject(request);
        if (!response.IsSuccess()) {
            // TODO If error is that it doesn't exist, return boost::none
            throw std::runtime_error("AWS exception in GetObject: " + response.GetError().GetExceptionName() + ": " + response.GetError().GetMessage());
        }

        Data data(response.GetResult().GetContentLength());
        response.GetResult().GetBody().read(static_cast<char*>(data.data()), data.size());

        return data;
    }

    uint64_t numBlocks() const {
        // TODO implement without listing all objects
        uint64_t result;
        forEachBlock([&result] (const BlockId&) {
            ++result;
        });
        return result;
    }

    void forEachBlock(std::function<void (const BlockId&)> callback) const {
        Aws::S3::Model::ListObjectsRequest request;
        request.WithBucket(bucket_name);
        auto response = _client.ListObjects(request);
        if (!response.IsSuccess()) {
            throw std::runtime_error("AWS exception in ListObject: " + response.GetError().GetExceptionName() + ": " + response.GetError().GetMessage());
        }
        // TODO Likely needs paging, see https://github.com/aws/aws-sdk-cpp/issues/369
        ASSERT(!response.GetResult().GetIsTruncated(), "paging?");

        for (const auto& object : response.GetResult().GetContents()) {
            callback(BlockId::FromString(object.GetKey()));
        }
    }

private:
    unique_ref<Aws::Client::ClientConfiguration> _clientConfig() {
        auto config = make_unique_ref<Aws::Client::ClientConfiguration>();
        config->region = region;
        return config;
    };

    S3APIInitializer _initializer;
    unique_ref<Aws::Client::ClientConfiguration> _config;
    Aws::S3::S3Client _client;
};

S3BlockStore2::S3BlockStore2() : _sdk(make_unique_ref<AwsS3SDK>()) {
}

S3BlockStore2::~S3BlockStore2() = default;

bool S3BlockStore2::tryCreate(const BlockId &blockId, const Data &data) {
    return _sdk->put(blockId, data, false);
}

bool S3BlockStore2::remove(const BlockId &blockId) {
    return _sdk->remove(blockId);
}

optional<Data> S3BlockStore2::load(const BlockId &blockId) const {
    return _sdk->load(blockId);
}

void S3BlockStore2::store(const BlockId &blockId, const Data &data) {
    _sdk->put(blockId, data, true);
}

uint64_t S3BlockStore2::numBlocks() const {
    return _sdk->numBlocks();
}

uint64_t S3BlockStore2::estimateNumFreeBytes() const {
    return 0;
}

uint64_t S3BlockStore2::blockSizeFromPhysicalBlockSize(uint64_t blockSize) const {
    return blockSize;
}

void S3BlockStore2::forEachBlock(std::function<void (const BlockId &)> callback) const {
    _sdk->forEachBlock(std::move(callback));
}

}
}
