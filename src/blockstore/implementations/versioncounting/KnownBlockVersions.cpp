#include <fstream>
#include <cpp-utils/random/Random.h>
#include "KnownBlockVersions.h"

namespace bf = boost::filesystem;
using std::unordered_map;
using std::pair;
using std::string;
using boost::optional;
using boost::none;
using cpputils::Random;

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

bool KnownBlockVersions::checkAndUpdateVersion(const Key &key, uint64_t version) {
    ASSERT(_valid, "Object not valid due to a std::move");

    uint64_t &found = _knownVersions[key]; // If the entry doesn't exist, this creates it with value 0.
    if (found > version) {
        return false;
    }

    found = version;
    return true;
}

void KnownBlockVersions::updateVersion(const Key &key, uint64_t version) {
    if (!checkAndUpdateVersion(key, version)) {
        throw std::logic_error("Tried to decrease block version");
    }
}

void KnownBlockVersions::_loadStateFile() {
    std::ifstream file(_stateFilePath.native().c_str());
    if (!file.good()) {
        // File doesn't exist means we loaded empty state. Assign a random client id.
        _myClientId = *reinterpret_cast<uint32_t*>(Random::PseudoRandom().getFixedSize<sizeof(uint32_t)>().data());
        return;
    }
    _checkHeader(&file);
    file.read((char*)&_myClientId, sizeof(_myClientId));
    ASSERT(file.good(), "Error reading file");
    uint64_t numEntries;
    file.read((char*)&numEntries, sizeof(numEntries));
    ASSERT(file.good(), "Error reading file");

    _knownVersions.clear();
    _knownVersions.reserve(static_cast<uint64_t>(1.2 * numEntries)); // Reserve for factor 1.2 more, so the file system doesn't immediately have to resize it on the first new block.
    for (uint64_t i = 0 ; i < numEntries; ++i) {
        auto entry = _readEntry(&file);
        _knownVersions.insert(entry);
    }

    _checkIsEof(&file);
};

void KnownBlockVersions::_checkHeader(std::ifstream *file) {
    char actualHeader[HEADER.size()];
    file->read(actualHeader, HEADER.size());
    ASSERT(file->good(), "Error reading file");
    if (HEADER != string(actualHeader, HEADER.size())) {
        throw std::runtime_error("Invalid local state: Invalid integrity file header.");
    }
}

pair<Key, uint64_t> KnownBlockVersions::_readEntry(std::ifstream *file) {
    pair<Key, uint64_t> result(Key::Null(), 0);

    file->read((char*)result.first.data(), result.first.BINARY_LENGTH);
    ASSERT(file->good(), "Error reading file");
    file->read((char*)&result.second, sizeof(result.second));
    ASSERT(file->good(), "Error reading file");

    return result;
};

void KnownBlockVersions::_checkIsEof(std::ifstream *file) {
    char dummy;
    file->read(&dummy, sizeof(dummy));
    if (!file->eof()) {
        throw std::runtime_error("There are more entries in the file than advertised");
    }
}

void KnownBlockVersions::_saveStateFile() const {
    std::ofstream file(_stateFilePath.native().c_str());
    file.write(HEADER.c_str(), HEADER.size());
    file.write((char*)&_myClientId, sizeof(_myClientId));
    uint64_t numEntries = _knownVersions.size();
    file.write((char*)&numEntries, sizeof(numEntries));
    for (const auto &entry : _knownVersions) {
        file.write((char*)entry.first.data(), entry.first.BINARY_LENGTH);
        file.write((char*)&entry.second, sizeof(entry.second));
    }
}

uint32_t KnownBlockVersions::myClientId() const {
    return _myClientId;
}

}
}
