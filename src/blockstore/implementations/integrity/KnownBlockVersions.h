#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_KNOWNBLOCKVERSIONS_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_KNOWNBLOCKVERSIONS_H_

#include <cpp-utils/macros.h>
#include <blockstore/utils/BlockId.h>
#include <boost/filesystem/path.hpp>
#include <boost/optional.hpp>
#include "ClientIdAndBlockId.h"
#include <cpp-utils/data/Deserializer.h>
#include <cpp-utils/data/Serializer.h>
#include <mutex>
#include <unordered_set>

namespace blockstore {
    namespace integrity {

        class KnownBlockVersions final {
        public:
            KnownBlockVersions(const boost::filesystem::path &stateFilePath, uint32_t myClientId);
            KnownBlockVersions(KnownBlockVersions &&rhs);
            ~KnownBlockVersions();

            __attribute__((warn_unused_result))
            bool checkAndUpdateVersion(uint32_t clientId, const BlockId &blockId, uint64_t version);

            uint64_t incrementVersion(const BlockId &blockId);

            void markBlockAsDeleted(const BlockId &blockId);

            bool blockShouldExist(const BlockId &blockId) const;
            std::unordered_set<BlockId> existingBlocks() const;

            uint64_t getBlockVersion(uint32_t clientId, const BlockId &blockId) const;

            uint32_t myClientId() const;
            const boost::filesystem::path &path() const;

            static constexpr uint32_t CLIENT_ID_FOR_DELETED_BLOCK = 0;

        private:
            std::unordered_map<ClientIdAndBlockId, uint64_t> _knownVersions;
            std::unordered_map<BlockId, uint32_t> _lastUpdateClientId; // The client who last updated the block

            boost::filesystem::path _stateFilePath;
            uint32_t _myClientId;
            mutable std::mutex _mutex;
            bool _valid;

            static const std::string HEADER;

            void _loadStateFile();
            void _saveStateFile() const;

            void _deserializeKnownVersions(cpputils::Deserializer *deserializer);
            void _serializeKnownVersions(cpputils::Serializer *serializer) const;

            static std::pair<ClientIdAndBlockId, uint64_t> _deserializeKnownVersionsEntry(cpputils::Deserializer *deserializer);
            static void _serializeKnownVersionsEntry(cpputils::Serializer *serializer, const std::pair<ClientIdAndBlockId, uint64_t> &entry);

            void _deserializeLastUpdateClientIds(cpputils::Deserializer *deserializer);
            void _serializeLastUpdateClientIds(cpputils::Serializer *serializer) const;

            static std::pair<BlockId, uint32_t> _deserializeLastUpdateClientIdEntry(cpputils::Deserializer *deserializer);
            static void _serializeLastUpdateClientIdEntry(cpputils::Serializer *serializer, const std::pair<BlockId, uint32_t> &entry);

            DISALLOW_COPY_AND_ASSIGN(KnownBlockVersions);
        };

    }
}


#endif
