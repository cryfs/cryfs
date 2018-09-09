#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_CLIENTIDANDBLOCKID_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_CLIENTIDANDBLOCKID_H_

#include <utility>
#include "blockstore/utils/BlockId.h"

namespace blockstore {
    namespace integrity {

        struct ClientIdAndBlockId final {
            ClientIdAndBlockId(uint32_t clientId_, BlockId blockId_): clientId(clientId_), blockId(blockId_) {}

            uint32_t clientId;
            BlockId blockId;
        };

    }
}

// Allow using it in std::unordered_set / std::unordered_map
namespace std {
    template<> struct hash<blockstore::integrity::ClientIdAndBlockId> {
        size_t operator()(const blockstore::integrity::ClientIdAndBlockId &ref) const {
            return std::hash<uint32_t>()(ref.clientId) ^ std::hash<blockstore::BlockId>()(ref.blockId);
        }
    };

    template<> struct equal_to<blockstore::integrity::ClientIdAndBlockId> {
        size_t operator()(const blockstore::integrity::ClientIdAndBlockId &lhs, const blockstore::integrity::ClientIdAndBlockId &rhs) const {
            return lhs.clientId == rhs.clientId && lhs.blockId == rhs.blockId;
        }
    };
}

#endif
