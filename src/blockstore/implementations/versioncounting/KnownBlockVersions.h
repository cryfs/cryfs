#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_KNOWNBLOCKVERSIONS_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_KNOWNBLOCKVERSIONS_H_

#include <cpp-utils/macros.h>
#include <blockstore/utils/Key.h>

namespace blockstore {
    namespace versioncounting {

        class KnownBlockVersions final {
        public:
            KnownBlockVersions();
            KnownBlockVersions(KnownBlockVersions &&rhs) = default;

            __attribute__((warn_unused_result))
            bool checkAndUpdateVersion(const Key &key, uint64_t version);

            void updateVersion(const Key &key, uint64_t version);

        private:
            std::unordered_map<Key, uint64_t> _knownVersions;
            DISALLOW_COPY_AND_ASSIGN(KnownBlockVersions);
        };

    }
}


#endif
