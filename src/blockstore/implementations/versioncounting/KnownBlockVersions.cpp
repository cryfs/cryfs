#include <fstream>
#include <cpp-utils/random/Random.h>
#include "KnownBlockVersions.h"

namespace bf = boost::filesystem;
using std::unordered_map;
using std::pair;
using std::string;
using std::unique_lock;
using std::mutex;
using boost::optional;
using boost::none;
using cpputils::Data;
using cpputils::Random;
using cpputils::Serializer;
using cpputils::Deserializer;

namespace blockstore {
namespace versioncounting {

const string KnownBlockVersions::HEADER = "cryfs.integritydata.knownblockversions;0";
constexpr uint32_t KnownBlockVersions::CLIENT_ID_FOR_DELETED_BLOCK;

KnownBlockVersions::KnownBlockVersions(const bf::path &stateFilePath)
        :_knownVersions(), _lastUpdateClientId(), _stateFilePath(stateFilePath), _myClientId(0), _mutex(), _valid(true) {
    unique_lock<mutex> lock(_mutex);
    _loadStateFile();
}

KnownBlockVersions::KnownBlockVersions(KnownBlockVersions &&rhs)
        : _knownVersions(), _lastUpdateClientId(), _stateFilePath(), _myClientId(0), _mutex(), _valid(true) {
    unique_lock<mutex> rhsLock(rhs._mutex);
    unique_lock<mutex> lock(_mutex);
    _knownVersions = std::move(rhs._knownVersions);
    _lastUpdateClientId = std::move(rhs._lastUpdateClientId);
    _stateFilePath = std::move(rhs._stateFilePath);
    _myClientId = rhs._myClientId;
    rhs._valid = false;
}

KnownBlockVersions::~KnownBlockVersions() {
    unique_lock<mutex> lock(_mutex);
    if (_valid) {
        _saveStateFile();
    }
}

bool KnownBlockVersions::checkAndUpdateVersion(uint32_t clientId, const Key &key, uint64_t version) {
    unique_lock<mutex> lock(_mutex);
    ASSERT(clientId != CLIENT_ID_FOR_DELETED_BLOCK, "This is not a valid client id");

    ASSERT(version > 0, "Version has to be >0"); // Otherwise we wouldn't handle notexisting entries correctly.
    ASSERT(_valid, "Object not valid due to a std::move");

    uint64_t &found = _knownVersions[{clientId, key}]; // If the entry doesn't exist, this creates it with value 0.
    if (found > version) {
        // This client already published a newer block version. Rollbacks are not allowed.
        return false;
    }

    uint32_t &lastUpdateClientId = _lastUpdateClientId[key]; // If entry doesn't exist, this creates it with value 0. However, in this case, found == 0 (and version > 0), which means found != version.
    if (found == version && lastUpdateClientId != clientId) {
        // This is a roll back to the "newest" block of client [clientId], which was since then superseded by a version from client _lastUpdateClientId[key].
        // This is not allowed.
        return false;
    }

    found = version;
    lastUpdateClientId = clientId;
    return true;
}

uint64_t KnownBlockVersions::incrementVersion(const Key &key, uint64_t lastVersion) {
    unique_lock<mutex> lock(_mutex);
    uint64_t &found = _knownVersions[{_myClientId, key}]; // If the entry doesn't exist, this creates it with value 0.
    uint64_t newVersion = std::max(lastVersion + 1, found + 1);
    if (newVersion == std::numeric_limits<uint64_t>::max()) {
        // It's *very* unlikely we ever run out of version numbers in 64bit...but just to be sure...
        throw std::runtime_error("Version overflow");
    }
    found = newVersion;
    _lastUpdateClientId[key] = _myClientId;
    return found;
}

void KnownBlockVersions::_loadStateFile() {
    optional<Data> file = Data::LoadFromFile(_stateFilePath);
    if (file == none) {
        // File doesn't exist means we loaded empty state. Assign a random client id.
        do {
            _myClientId = *reinterpret_cast<uint32_t*>(Random::PseudoRandom().getFixedSize<sizeof(uint32_t)>().data());
        } while(_myClientId == CLIENT_ID_FOR_DELETED_BLOCK); // Safety check - CLIENT_ID_FOR_DELETED_BLOCK shouldn't be used by any valid client.
        return;
    }

    Deserializer deserializer(&*file);
    if (HEADER != deserializer.readString()) {
        throw std::runtime_error("Invalid local state: Invalid integrity file header.");
    }
    _myClientId = deserializer.readUint32();
    _deserializeKnownVersions(&deserializer);
    _deserializeLastUpdateClientIds(&deserializer);

    deserializer.finished();
};


void KnownBlockVersions::_saveStateFile() const {
    Serializer serializer(
            Serializer::StringSize(HEADER) + sizeof(uint32_t) +
            sizeof(uint64_t) + _knownVersions.size() * (sizeof(uint32_t) + Key::BINARY_LENGTH + sizeof(uint64_t)) +
            sizeof(uint64_t) + _lastUpdateClientId.size() * (Key::BINARY_LENGTH + sizeof(uint32_t)));
    serializer.writeString(HEADER);
    serializer.writeUint32(_myClientId);
    _serializeKnownVersions(&serializer);
    _serializeLastUpdateClientIds(&serializer);

    serializer.finished().StoreToFile(_stateFilePath);
}

void KnownBlockVersions::_deserializeKnownVersions(Deserializer *deserializer) {
    uint64_t numEntries = deserializer->readUint64();
    _knownVersions.clear();
    _knownVersions.reserve(static_cast<uint64_t>(1.2 * numEntries)); // Reserve for factor 1.2 more, so the file system doesn't immediately have to resize it on the first new block.
    for (uint64_t i = 0 ; i < numEntries; ++i) {
        auto entry = _deserializeKnownVersionsEntry(deserializer);
        _knownVersions.insert(entry);
    }
}

void KnownBlockVersions::_serializeKnownVersions(Serializer *serializer) const {
    uint64_t numEntries = _knownVersions.size();
    serializer->writeUint64(numEntries);

    for (const auto &entry : _knownVersions) {
        _serializeKnownVersionsEntry(serializer, entry);
    }
}

pair<ClientIdAndBlockKey, uint64_t> KnownBlockVersions::_deserializeKnownVersionsEntry(Deserializer *deserializer) {
    uint32_t clientId = deserializer->readUint32();
    Key blockKey = deserializer->readFixedSizeData<Key::BINARY_LENGTH>();
    uint64_t version = deserializer->readUint64();

    return {{clientId, blockKey}, version};
};

void KnownBlockVersions::_serializeKnownVersionsEntry(Serializer *serializer, const pair<ClientIdAndBlockKey, uint64_t> &entry) {
    serializer->writeUint32(entry.first.clientId);
    serializer->writeFixedSizeData<Key::BINARY_LENGTH>(entry.first.blockKey);
    serializer->writeUint64(entry.second);
}

void KnownBlockVersions::_deserializeLastUpdateClientIds(Deserializer *deserializer) {
    uint64_t numEntries = deserializer->readUint64();
    _lastUpdateClientId.clear();
    _lastUpdateClientId.reserve(static_cast<uint64_t>(1.2 * numEntries)); // Reserve for factor 1.2 more, so the file system doesn't immediately have to resize it on the first new block.
    for (uint64_t i = 0 ; i < numEntries; ++i) {
        auto entry = _deserializeLastUpdateClientIdEntry(deserializer);
        _lastUpdateClientId.insert(entry);
    }
}

void KnownBlockVersions::_serializeLastUpdateClientIds(Serializer *serializer) const {
    uint64_t numEntries = _lastUpdateClientId.size();
    serializer->writeUint64(numEntries);

    for (const auto &entry : _lastUpdateClientId) {
        _serializeLastUpdateClientIdEntry(serializer, entry);
    }
}

pair<Key, uint32_t> KnownBlockVersions::_deserializeLastUpdateClientIdEntry(Deserializer *deserializer) {
    Key blockKey = deserializer->readFixedSizeData<Key::BINARY_LENGTH>();
    uint32_t clientId = deserializer->readUint32();

    return {blockKey, clientId};
};

void KnownBlockVersions::_serializeLastUpdateClientIdEntry(Serializer *serializer, const pair<Key, uint32_t> &entry) {
    serializer->writeFixedSizeData<Key::BINARY_LENGTH>(entry.first);
    serializer->writeUint32(entry.second);
}

uint32_t KnownBlockVersions::myClientId() const {
    return _myClientId;
}

uint64_t KnownBlockVersions::getBlockVersion(uint32_t clientId, const Key &key) const {
    unique_lock<mutex> lock(_mutex);
    return _knownVersions.at({clientId, key});
}

void KnownBlockVersions::markBlockAsDeleted(const Key &key) {
    _lastUpdateClientId[key] = CLIENT_ID_FOR_DELETED_BLOCK;
}

bool KnownBlockVersions::blockShouldExist(const Key &key) const {
    auto found = _lastUpdateClientId.find(key);
    if (found == _lastUpdateClientId.end()) {
        // We've never seen (i.e. loaded) this block. So we can't say it has to exist.
        return false;
    }
    // We've seen the block before. If we didn't delete it, it should exist (only works for single-client scenario).
    return found->second != CLIENT_ID_FOR_DELETED_BLOCK;
}

}
}
