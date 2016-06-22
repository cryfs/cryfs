#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_KNOWNBLOCKVERSIONS_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_KNOWNBLOCKVERSIONS_H_

#include <cpp-utils/macros.h>
#include <blockstore/utils/Key.h>
#include <boost/filesystem/path.hpp>
#include <boost/optional.hpp>
#include "ClientIdAndBlockKey.h"
#include <cpp-utils/data/Deserializer.h>

namespace blockstore {
    namespace versioncounting {

        class KnownBlockVersions final {
        public:
            KnownBlockVersions(const boost::filesystem::path &stateFilePath);
            KnownBlockVersions(KnownBlockVersions &&rhs);
            ~KnownBlockVersions();

            __attribute__((warn_unused_result))
            bool checkAndUpdateVersion(uint32_t clientId, const Key &key, uint64_t version);

            void updateVersion(const Key &key, uint64_t version);

            uint32_t myClientId() const;

        private:
            std::unordered_map<ClientIdAndBlockKey, uint64_t> _knownVersions;
            boost::filesystem::path _stateFilePath;
            uint32_t _myClientId;
            bool _valid;

            static const std::string HEADER;

            void _loadStateFile();
            static std::pair<ClientIdAndBlockKey, uint64_t> _readEntry(cpputils::Deserializer *deserializer);
            void _saveStateFile() const;

            DISALLOW_COPY_AND_ASSIGN(KnownBlockVersions);
        };

    }
}


#endif
