#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_KNOWNBLOCKVERSIONS_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_KNOWNBLOCKVERSIONS_H_

#include <cpp-utils/macros.h>
#include <blockstore/utils/Key.h>
#include <boost/filesystem/path.hpp>
#include <boost/optional.hpp>
#include "ClientIdAndBlockKey.h"
#include <cpp-utils/data/Deserializer.h>
#include <cpp-utils/data/Serializer.h>
#include <mutex>

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
            std::unordered_map<Key, uint32_t> _lastUpdateClientId; // The client who last updated the block

            boost::filesystem::path _stateFilePath;
            uint32_t _myClientId;
            mutable std::mutex _mutex;
            bool _valid;

            static const std::string HEADER;

            void _loadStateFile();
            void _saveStateFile() const;

            void _deserializeKnownVersions(cpputils::Deserializer *deserializer);
            void _serializeKnownVersions(cpputils::Serializer *serializer) const;

            static std::pair<ClientIdAndBlockKey, uint64_t> _deserializeKnownVersionsEntry(cpputils::Deserializer *deserializer);
            static void _serializeKnownVersionsEntry(cpputils::Serializer *serializer, const std::pair<ClientIdAndBlockKey, uint64_t> &entry);

            void _deserializeLastUpdateClientIds(cpputils::Deserializer *deserializer);
            void _serializeLastUpdateClientIds(cpputils::Serializer *serializer) const;

            static std::pair<Key, uint32_t> _deserializeLastUpdateClientIdEntry(cpputils::Deserializer *deserializer);
            static void _serializeLastUpdateClientIdEntry(cpputils::Serializer *serializer, const std::pair<Key, uint32_t> &entry);

            DISALLOW_COPY_AND_ASSIGN(KnownBlockVersions);
        };

    }
}


#endif
