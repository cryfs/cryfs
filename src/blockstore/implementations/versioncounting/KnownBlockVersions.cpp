#include <fstream>
#include <cpp-utils/random/Random.h>
#include "KnownBlockVersions.h"
#include <cpp-utils/data/Serializer.h>
#include <cpp-utils/data/Deserializer.h>

namespace bf = boost::filesystem;
using std::unordered_map;
using std::pair;
using std::string;
using boost::optional;
using boost::none;
using cpputils::Data;
using cpputils::Random;
using cpputils::Serializer;
using cpputils::Deserializer;

namespace blockstore {
namespace versioncounting {

const string KnownBlockVersions::HEADER = "cryfs.integritydata.knownblockversions;0";

KnownBlockVersions::KnownBlockVersions(const bf::path &stateFilePath)
        :_knownVersions(), _stateFilePath(stateFilePath), _myClientId(0), _valid(true) {
    _loadStateFile();
}

KnownBlockVersions::KnownBlockVersions(KnownBlockVersions &&rhs)
        : _knownVersions(std::move(rhs._knownVersions)), _stateFilePath(std::move(rhs._stateFilePath)), _myClientId(rhs._myClientId), _valid(true) {
    rhs._valid = false;
}

KnownBlockVersions::~KnownBlockVersions() {
    if (_valid) {
        _saveStateFile();
    }
}

bool KnownBlockVersions::checkAndUpdateVersion(uint32_t clientId, const Key &key, uint64_t version) {
    ASSERT(_valid, "Object not valid due to a std::move");

    uint64_t &found = _knownVersions[{clientId, key}]; // If the entry doesn't exist, this creates it with value 0.
    if (found > version) {
        return false;
    }

    found = version;
    return true;
}

void KnownBlockVersions::updateVersion(const Key &key, uint64_t version) {
    if (!checkAndUpdateVersion(_myClientId, key, version)) {
        throw std::logic_error("Tried to decrease block version");
    }
}

void KnownBlockVersions::_loadStateFile() {
    optional<Data> file = Data::LoadFromFile(_stateFilePath);
    if (file == none) {
        // File doesn't exist means we loaded empty state. Assign a random client id.
        _myClientId = *reinterpret_cast<uint32_t*>(Random::PseudoRandom().getFixedSize<sizeof(uint32_t)>().data());
        return;
    }

    Deserializer deserializer(&*file);
    if (HEADER != deserializer.readString()) {
        throw std::runtime_error("Invalid local state: Invalid integrity file header.");
    }
    _myClientId = deserializer.readUint32();
    uint64_t numEntries = deserializer.readUint64();

    _knownVersions.clear();
    _knownVersions.reserve(static_cast<uint64_t>(1.2 * numEntries)); // Reserve for factor 1.2 more, so the file system doesn't immediately have to resize it on the first new block.
    for (uint64_t i = 0 ; i < numEntries; ++i) {
        auto entry = _readEntry(&deserializer);
        _knownVersions.insert(entry);
    }

    deserializer.finished();
};

pair<ClientIdAndBlockKey, uint64_t> KnownBlockVersions::_readEntry(Deserializer *deserializer) {
    uint32_t clientId = deserializer->readUint32();
    Key blockKey = deserializer->readFixedSizeData<Key::BINARY_LENGTH>();
    uint64_t version = deserializer->readUint64();

    return {{clientId, blockKey}, version};
};

void KnownBlockVersions::_saveStateFile() const {
    uint64_t numEntries = _knownVersions.size();

    Serializer serializer(Serializer::StringSize(HEADER) + sizeof(uint32_t) + sizeof(uint64_t) + numEntries * (sizeof(uint32_t) + Key::BINARY_LENGTH + sizeof(uint64_t)));
    serializer.writeString(HEADER);
    serializer.writeUint32(_myClientId);
    serializer.writeUint64(numEntries);

    for (const auto &entry : _knownVersions) {
        serializer.writeUint32(entry.first.clientId);
        serializer.writeFixedSizeData<Key::BINARY_LENGTH>(entry.first.blockKey);
        serializer.writeUint64(entry.second);
    }

    serializer.finished().StoreToFile(_stateFilePath);
}

uint32_t KnownBlockVersions::myClientId() const {
    return _myClientId;
}

}
}
