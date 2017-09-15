#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_CLIENTIDANDBLOCKKEY_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_INTEGRITY_CLIENTIDANDBLOCKKEY_H_

#include <utility>
#include "../../utils/Key.h"

namespace blockstore {
    namespace integrity {

        struct ClientIdAndBlockKey {
            uint32_t clientId;
            Key blockKey;
        };

    }
}

// Allow using it in std::unordered_set / std::unordered_map
namespace std {
    template<> struct hash<blockstore::integrity::ClientIdAndBlockKey> {
        size_t operator()(const blockstore::integrity::ClientIdAndBlockKey &ref) const {
            return std::hash<uint32_t>()(ref.clientId) ^ std::hash<blockstore::Key>()(ref.blockKey);
        }
    };

    template<> struct equal_to<blockstore::integrity::ClientIdAndBlockKey> {
        size_t operator()(const blockstore::integrity::ClientIdAndBlockKey &lhs, const blockstore::integrity::ClientIdAndBlockKey &rhs) const {
            return lhs.clientId == rhs.clientId && lhs.blockKey == rhs.blockKey;
        }
    };
}

#endif
