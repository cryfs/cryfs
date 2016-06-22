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

    _knownVersions.clear();
    optional<pair<Key, uint64_t>> entry = _readEntry(&file);
    while(none != entry) {
        _knownVersions.insert(*entry);
        entry = _readEntry(&file);
    }
    ASSERT(file.eof(), "Didn't read until end of file");
};

void KnownBlockVersions::_checkHeader(std::ifstream *file) {
    char actualHeader[HEADER.size()];
    file->read(actualHeader, HEADER.size());
    if (HEADER != string(actualHeader, HEADER.size())) {
        throw std::runtime_error("Invalid local state: Invalid integrity file header.");
    }
}

optional<pair<Key, uint64_t>> KnownBlockVersions::_readEntry(std::ifstream *file) {
    ASSERT(file->good(), "Error reading file");
    pair<Key, uint64_t> result(Key::Null(), 0);

    file->read((char*)result.first.data(), result.first.BINARY_LENGTH);
    if (file->eof()) {
        // Couldn't read another entry. File end.
        return none;
    }
    ASSERT(file->good(), "Error reading file");
    file->read((char*)&result.second, sizeof(result.second));
    ASSERT(file->good(), "Error reading file");

    return result;
};

void KnownBlockVersions::_saveStateFile() const {
    std::ofstream file(_stateFilePath.native().c_str());
    file.write(HEADER.c_str(), HEADER.size());
    file.write((char*)&_myClientId, sizeof(_myClientId));
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
