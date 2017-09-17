#include "MockBlock.h"
#include "MockBlockStore.h"

namespace blockstore {
    namespace mock {

        void MockBlock::write(const void *source, uint64_t offset, uint64_t size) {
            _blockStore->_increaseNumWrittenBlocks(blockId());
            return _baseBlock->write(source, offset, size);
        }

        void MockBlock::resize(size_t newSize) {
            _blockStore->_increaseNumResizedBlocks(blockId());
            return _baseBlock->resize(newSize);
        }

    }
}
