#include "VersionCountingBlock.h"

namespace blockstore {
    namespace versioncounting {
        constexpr unsigned int VersionCountingBlock::CLIENTID_HEADER_OFFSET;
        constexpr unsigned int VersionCountingBlock::VERSION_HEADER_OFFSET;
        constexpr unsigned int VersionCountingBlock::HEADER_LENGTH;
        constexpr uint16_t VersionCountingBlock::FORMAT_VERSION_HEADER;
        constexpr uint64_t VersionCountingBlock::VERSION_ZERO;

#ifndef CRYFS_NO_COMPATIBILITY
        void VersionCountingBlock::migrateFromBlockstoreWithoutVersionNumbers(cpputils::unique_ref<Block> baseBlock, KnownBlockVersions *knownBlockVersions) {
            uint64_t version = knownBlockVersions->incrementVersion(baseBlock->key(), VERSION_ZERO);

            cpputils::Data data(baseBlock->size());
            std::memcpy(data.data(), baseBlock->data(), data.size());
            cpputils::Data dataWithHeader = _prependHeaderToData(knownBlockVersions->myClientId(), version, std::move(data));
            baseBlock->resize(dataWithHeader.size());
            baseBlock->write(dataWithHeader.data(), 0, dataWithHeader.size());
        }
#endif
    }
}
