#include "KnownBlockVersions.h"

namespace blockstore {
namespace versioncounting {

KnownBlockVersions::KnownBlockVersions()
        :_knownVersions() {
}

bool KnownBlockVersions::checkAndUpdateVersion(const Key &key, uint64_t version) {
    auto found = _knownVersions.find(key);
    if (found == _knownVersions.end()) {
        _knownVersions.emplace(key, version);
        return true;
    }

    if (found->second > version) {
        return false;
    }

    found->second = version;
    return true;
}

void KnownBlockVersions::updateVersion(const Key &key, uint64_t version) {
    if (!checkAndUpdateVersion(key, version)) {
        throw std::logic_error("Tried to decrease block version");
    }
}

}
}
