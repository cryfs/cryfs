#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_MOCK_MOCKBLOCK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_MOCK_MOCKBLOCK_H_

#include <blockstore/interface/Block.h>
#include <cpp-utils/pointer/unique_ref.h>

namespace blockstore {
    namespace mock {

        class MockBlockStore;

        class MockBlock final : public blockstore::Block {
        public:
            MockBlock(cpputils::unique_ref<blockstore::Block> baseBlock, MockBlockStore *blockStore)
                    :Block(baseBlock->blockId()), _baseBlock(std::move(baseBlock)), _blockStore(blockStore) {
            }

            const void *data() const override {
              return _baseBlock->data();
            }

            void write(const void *source, uint64_t offset, uint64_t size) override;

            size_t size() const override {
              return _baseBlock->size();
            }

            void resize(size_t newSize) override;

            cpputils::unique_ref<blockstore::Block> releaseBaseBlock() {
              return std::move(_baseBlock);
            }

        private:
            cpputils::unique_ref<blockstore::Block> _baseBlock;
            MockBlockStore *_blockStore;
            friend class MockBlockStore;

            DISALLOW_COPY_AND_ASSIGN(MockBlock);
        };


    }
}

#endif
