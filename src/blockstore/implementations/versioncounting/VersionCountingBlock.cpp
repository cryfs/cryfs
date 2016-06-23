#include "VersionCountingBlock.h"

namespace blockstore {
    namespace versioncounting {
        constexpr unsigned int VersionCountingBlock::CLIENTID_HEADER_OFFSET;
        constexpr unsigned int VersionCountingBlock::VERSION_HEADER_OFFSET;
        constexpr unsigned int VersionCountingBlock::HEADER_LENGTH;
        constexpr uint16_t VersionCountingBlock::FORMAT_VERSION_HEADER;
        constexpr uint64_t VersionCountingBlock::VERSION_ZERO;
        constexpr uint64_t VersionCountingBlock::VERSION_DELETED;
    }
}
