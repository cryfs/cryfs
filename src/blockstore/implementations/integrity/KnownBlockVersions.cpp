#include <fstream>
#include <cpp-utils/random/Random.h>
#include <unordered_set>
#include "KnownBlockVersions.h"

namespace bf = boost::filesystem;
using std::pair;
using std::string;
using std::unique_lock;
using std::mutex;
using boost::optional;
using boost::none;
using cpputils::Data;
using cpputils::Serializer;
using cpputils::Deserializer;

namespace blockstore {
namespace integrity {

const string KnownBlockVersions::OLD_HEADER = "cryfs.integritydata.knownblockversions;0";
const string KnownBlockVersions::HEADER = "cryfs.integritydata.knownblockversions;1";
constexpr uint32_t KnownBlockVersions::CLIENT_ID_FOR_DELETED_BLOCK;

KnownBlockVersions::KnownBlockVersions(const bf::path &stateFilePath, uint32_t myClientId)
        :_integrityViolationOnPreviousRun(false), _knownVersions(), _lastUpdateClientId(), _stateFilePath(stateFilePath), _myClientId(myClientId), _mutex(), _valid(true) {
    const unique_lock<mutex> lock(_mutex);
    ASSERT(_myClientId != CLIENT_ID_FOR_DELETED_BLOCK, "This is not a valid client id");
    _loadStateFile();
}

KnownBlockVersions::KnownBlockVersions(KnownBlockVersions &&rhs) // NOLINT (intentionally not noexcept)
        : _integrityViolationOnPreviousRun(false), _knownVersions(), _lastUpdateClientId(), _stateFilePath(), _myClientId(0), _mutex(), _valid(true) {
    const unique_lock<mutex> rhsLock(rhs._mutex);
    const unique_lock<mutex> lock(_mutex);
    // NOLINTBEGIN(cppcoreguidelines-prefer-member-initializer) -- we need to initialize those within the mutexes
    _integrityViolationOnPreviousRun = rhs._integrityViolationOnPreviousRun;
    _knownVersions = std::move(rhs._knownVersions);
    _lastUpdateClientId = std::move(rhs._lastUpdateClientId);
    _stateFilePath = std::move(rhs._stateFilePath);
    _myClientId = rhs._myClientId;
    rhs._valid = false;
    // NOLINTEND(cppcoreguidelines-prefer-member-initializer)
}

KnownBlockVersions::~KnownBlockVersions() {
    const unique_lock<mutex> lock(_mutex);
    if (_valid) {
        _saveStateFile();
    }
}

void KnownBlockVersions::setIntegrityViolationOnPreviousRun(bool value) {
    _integrityViolationOnPreviousRun = value;
}

bool KnownBlockVersions::integrityViolationOnPreviousRun() const {
    return _integrityViolationOnPreviousRun;
}

bool KnownBlockVersions::checkAndUpdateVersion(uint32_t clientId, const BlockId &blockId, uint64_t version) {
    const unique_lock<mutex> lock(_mutex);
    ASSERT(clientId != CLIENT_ID_FOR_DELETED_BLOCK, "This is not a valid client id");

    ASSERT(version > 0, "Version has to be >0"); // Otherwise we wouldn't handle notexisting entries correctly.
    ASSERT(_valid, "Object not valid due to a std::move");

    uint64_t &found = _knownVersions[{clientId, blockId}]; // If the entry doesn't exist, this creates it with value 0.
    if (found > version) {
        // This client already published a newer block version. Rollbacks are not allowed.
        return false;
    }

    uint32_t &lastUpdateClientId = _lastUpdateClientId[blockId]; // If entry doesn't exist, this creates it with value 0. However, in this case, found == 0 (and version > 0), which means found != version.
    if (found == version && lastUpdateClientId != clientId) {
        // This is a roll back to the "newest" block of client [clientId], which was since then superseded by a version from client _lastUpdateClientId[blockId].
        // This is not allowed.
        return false;
    }

    found = version;
    lastUpdateClientId = clientId;
    return true;
}

uint64_t KnownBlockVersions::incrementVersion(const BlockId &blockId) {
    const unique_lock<mutex> lock(_mutex);
    uint64_t &found = _knownVersions[{_myClientId, blockId}]; // If the entry doesn't exist, this creates it with value 0.
    const uint64_t newVersion = found + 1;
    if (newVersion == std::numeric_limits<uint64_t>::max()) {
        // It's *very* unlikely we ever run out of version numbers in 64bit...but just to be sure...
        throw std::runtime_error("Version overflow");
    }
    found = newVersion;
    _lastUpdateClientId[blockId] = _myClientId;
    return found;
}

void KnownBlockVersions::_loadStateFile() {
    optional<Data> file = Data::LoadFromFile(_stateFilePath);
    if (file == none) {
        // File doesn't exist means we loaded empty state.
        return;
    }
    Deserializer deserializer(&*file);
    const string loaded_header = deserializer.readString();

#ifndef CRYFS_NO_COMPATIBILITY
    if (OLD_HEADER == loaded_header) {
        _knownVersions = _deserializeKnownVersions(&deserializer);
        _lastUpdateClientId = _deserializeLastUpdateClientIds(&deserializer);

        deserializer.finished();
        _saveStateFile();
        return;
    }
#endif
    if (HEADER != loaded_header) {
        throw std::runtime_error("Invalid local state: Invalid integrity file header.");
    }
    _integrityViolationOnPreviousRun = deserializer.readBool();
    _knownVersions = _deserializeKnownVersions(&deserializer);
    _lastUpdateClientId = _deserializeLastUpdateClientIds(&deserializer);

    deserializer.finished();
};


void KnownBlockVersions::_saveStateFile() const {
    Serializer serializer(
            Serializer::StringSize(HEADER) +
            Serializer::BoolSize() +
            sizeof(uint64_t) + _knownVersions.size() * (sizeof(uint32_t) + BlockId::BINARY_LENGTH + sizeof(uint64_t)) +
            sizeof(uint64_t) + _lastUpdateClientId.size() * (BlockId::BINARY_LENGTH + sizeof(uint32_t)));
    serializer.writeString(HEADER);
    serializer.writeBool(_integrityViolationOnPreviousRun);
    _serializeKnownVersions(&serializer, _knownVersions);
    _serializeLastUpdateClientIds(&serializer, _lastUpdateClientId);

    serializer.finished().StoreToFile(_stateFilePath);
}

std::unordered_map<ClientIdAndBlockId, uint64_t> KnownBlockVersions::_deserializeKnownVersions(Deserializer *deserializer) {
    const uint64_t numEntries = deserializer->readUint64();
    std::unordered_map<ClientIdAndBlockId, uint64_t> result;
    result.reserve(static_cast<uint64_t>(1.2 * numEntries)); // Reserve for factor 1.2 more, so the file system doesn't immediately have to resize it on the first new block.
    for (uint64_t i = 0 ; i < numEntries; ++i) {
        auto entry = _deserializeKnownVersionsEntry(deserializer);
        result.insert(entry);
    }

    return result;
}

void KnownBlockVersions::_serializeKnownVersions(Serializer *serializer, const std::unordered_map<ClientIdAndBlockId, uint64_t>& knownVersions) {
    const uint64_t numEntries = knownVersions.size();
    serializer->writeUint64(numEntries);

    for (const auto &entry : knownVersions) {
        _serializeKnownVersionsEntry(serializer, entry);
    }
}

pair<ClientIdAndBlockId, uint64_t> KnownBlockVersions::_deserializeKnownVersionsEntry(Deserializer *deserializer) {
    const uint32_t clientId = deserializer->readUint32();
    const BlockId blockId(deserializer->readFixedSizeData<BlockId::BINARY_LENGTH>());
    const uint64_t version = deserializer->readUint64();

    return {{clientId, blockId}, version};
};

void KnownBlockVersions::_serializeKnownVersionsEntry(Serializer *serializer, const pair<ClientIdAndBlockId, uint64_t> &entry) {
    serializer->writeUint32(entry.first.clientId);
    serializer->writeFixedSizeData<BlockId::BINARY_LENGTH>(entry.first.blockId.data());
    serializer->writeUint64(entry.second);
}

std::unordered_map<BlockId, uint32_t> KnownBlockVersions::_deserializeLastUpdateClientIds(Deserializer *deserializer) {
    const uint64_t numEntries = deserializer->readUint64();
    std::unordered_map<BlockId, uint32_t> result;
    result.reserve(static_cast<uint64_t>(1.2 * numEntries)); // Reserve for factor 1.2 more, so the file system doesn't immediately have to resize it on the first new block.
    for (uint64_t i = 0 ; i < numEntries; ++i) {
        auto entry = _deserializeLastUpdateClientIdEntry(deserializer);
        result.insert(entry);
    }
    return result;
}

void KnownBlockVersions::_serializeLastUpdateClientIds(Serializer *serializer, const std::unordered_map<BlockId, uint32_t>& lastUpdateClientId) {
    const uint64_t numEntries = lastUpdateClientId.size();
    serializer->writeUint64(numEntries);

    for (const auto &entry : lastUpdateClientId) {
        _serializeLastUpdateClientIdEntry(serializer, entry);
    }
}

pair<BlockId, uint32_t> KnownBlockVersions::_deserializeLastUpdateClientIdEntry(Deserializer *deserializer) {
    const BlockId blockId(deserializer->readFixedSizeData<BlockId::BINARY_LENGTH>());
    const uint32_t clientId = deserializer->readUint32();

    return {blockId, clientId};
};

void KnownBlockVersions::_serializeLastUpdateClientIdEntry(Serializer *serializer, const pair<BlockId, uint32_t> &entry) {
    serializer->writeFixedSizeData<BlockId::BINARY_LENGTH>(entry.first.data());
    serializer->writeUint32(entry.second);
}

uint32_t KnownBlockVersions::myClientId() const {
    return _myClientId;
}

uint64_t KnownBlockVersions::getBlockVersion(uint32_t clientId, const BlockId &blockId) const {
    const unique_lock<mutex> lock(_mutex);
    return _knownVersions.at({clientId, blockId});
}

void KnownBlockVersions::markBlockAsDeleted(const BlockId &blockId) {
    _lastUpdateClientId[blockId] = CLIENT_ID_FOR_DELETED_BLOCK;
}

bool KnownBlockVersions::blockShouldExist(const BlockId &blockId) const {
    auto found = _lastUpdateClientId.find(blockId);
    if (found == _lastUpdateClientId.end()) {
        // We've never seen (i.e. loaded) this block. So we can't say it has to exist.
        return false;
    }
    // We've seen the block before. If we didn't delete it, it should exist (only works for single-client scenario).
    return found->second != CLIENT_ID_FOR_DELETED_BLOCK;
}

std::unordered_set<BlockId> KnownBlockVersions::existingBlocks() const {
    std::unordered_set<BlockId> result;
    for (const auto &entry : _lastUpdateClientId) {
        if (entry.second != CLIENT_ID_FOR_DELETED_BLOCK) {
            result.insert(entry.first);
        }
    }
    return result;
}

const bf::path &KnownBlockVersions::path() const {
    return _stateFilePath;
}

}
}
