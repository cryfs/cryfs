#pragma once
#ifndef MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_MOCKBLOCKSTORE_H_
#define MESSMER_BLOBSTORE_TEST_IMPLEMENTATIONS_ONBLOCKS_DATATREESTORE_GROWING_TESTUTILS_MOCKBLOCKSTORE_H_

#include <gmock/gmock.h>
#include <blockstore/implementations/testfake/FakeBlockStore.h>
#include <mutex>

class MockBlockStore final : public blockstore::BlockStore {
public:
    MockBlockStore(cpputils::unique_ref<BlockStore> baseBlockStore = cpputils::make_unique_ref<blockstore::testfake::FakeBlockStore>())
        : _baseBlockStore(std::move(baseBlockStore)) {
    }

    blockstore::Key createKey() override {
        return _baseBlockStore->createKey();
    }

    boost::optional<cpputils::unique_ref<blockstore::Block>> tryCreate(const blockstore::Key &key, cpputils::Data data) override {
        return _baseBlockStore->tryCreate(key, std::move(data));
    }

    boost::optional<cpputils::unique_ref<blockstore::Block>> load(const blockstore::Key &key) override {
        {
            std::unique_lock<std::mutex> lock(_mutex);
            loadedBlocks.push_back(key);
        }
        return _baseBlockStore->load(key);
    }

    void remove(const blockstore::Key &key) override {
        return _baseBlockStore->remove(key);
    }

    uint64_t numBlocks() const override {
        return _baseBlockStore->numBlocks();
    }

    uint64_t estimateNumFreeBytes() const override {
        return _baseBlockStore->estimateNumFreeBytes();
    }

    uint64_t blockSizeFromPhysicalBlockSize(uint64_t blockSize) const override {
        return _baseBlockStore->blockSizeFromPhysicalBlockSize(blockSize);
    }

    void forEachBlock(std::function<void(const blockstore::Key &)> callback) const override {
        return _baseBlockStore->forEachBlock(callback);
    }

    void remove(cpputils::unique_ref<blockstore::Block> block) override {
        return _baseBlockStore->remove(std::move(block));
    }

    std::vector<blockstore::Key> loadedBlocks;

private:
    std::mutex _mutex;
    cpputils::unique_ref<blockstore::BlockStore> _baseBlockStore;
};

#endif
