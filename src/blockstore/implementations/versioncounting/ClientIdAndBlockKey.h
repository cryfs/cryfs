#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_CLIENTIDANDBLOCKKEY_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_CLIENTIDANDBLOCKKEY_H_

#include <utility>

namespace blockstore {
    namespace versioncounting {

        struct ClientIdAndBlockKey {
            uint32_t clientId;
            Key blockKey;
        };

    }
}

// Allow using it in std::unordered_set / std::unordered_map
namespace std {
    template<> struct hash<blockstore::versioncounting::ClientIdAndBlockKey> {
        size_t operator()(const blockstore::versioncounting::ClientIdAndBlockKey &ref) const {
            return std::hash<uint32_t>()(ref.clientId) ^ std::hash<blockstore::Key>()(ref.blockKey);
        }
    };

    template<> struct equal_to<blockstore::versioncounting::ClientIdAndBlockKey> {
        size_t operator()(const blockstore::versioncounting::ClientIdAndBlockKey &lhs, const blockstore::versioncounting::ClientIdAndBlockKey &rhs) const {
            return lhs.clientId == rhs.clientId && lhs.blockKey == rhs.blockKey;
        }
    };
}

#endif
