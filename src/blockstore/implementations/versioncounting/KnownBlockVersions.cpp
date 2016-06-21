#include <fstream>
#include "KnownBlockVersions.h"

namespace bf = boost::filesystem;
using std::unordered_map;
using std::pair;
using std::string;
using boost::optional;
using boost::none;

namespace blockstore {
namespace versioncounting {

const string KnownBlockVersions::HEADER = "cryfs.integritydata.knownblockversions;0\0";

KnownBlockVersions::KnownBlockVersions(const bf::path &stateFilePath)
        :_knownVersions(_loadStateFile(stateFilePath)), _stateFilePath(stateFilePath), _valid(true) {
}

KnownBlockVersions::KnownBlockVersions(KnownBlockVersions &&rhs)
        : _knownVersions(std::move(rhs._knownVersions)), _stateFilePath(std::move(rhs._stateFilePath)), _valid(true) {
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

unordered_map<Key, uint64_t> KnownBlockVersions::_loadStateFile(const bf::path &stateFilePath) {
    std::ifstream file(stateFilePath.native().c_str());
    if (!file.good()) {
        return unordered_map<Key, uint64_t>();
    }
    _checkHeader(&file);

    unordered_map<Key, uint64_t> result;
    optional<pair<Key, uint64_t>> entry = _readEntry(&file);
    while(none != entry) {
        result.insert(*entry);
        entry = _readEntry(&file);
    }
    ASSERT(file.eof(), "Didn't read until end of file");
    return result;
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
    for (const auto &entry : _knownVersions) {
        file.write((char*)entry.first.data(), entry.first.BINARY_LENGTH);
        file.write((char*)&entry.second, sizeof(entry.second));
    }
}

}
}
