#include "DirEntryList.h"

//TODO Get rid of that in favor of better error handling
#include <messmer/fspp/fuse/FuseErrnoException.h>

using cpputils::Data;
using std::string;
using std::vector;
using blockstore::Key;

namespace cryfs {
namespace fsblobstore {

Data DirEntryList::serialize() const {
    Data serialized(_serializedSize());
    unsigned int offset = 0;
    for (const auto &entry : _entries) {
        entry.serialize(static_cast<uint8_t*>(serialized.dataOffset(offset)));
        offset += entry.serializedSize();
    }
    return serialized;
}

uint64_t DirEntryList::_serializedSize() const {
    uint64_t serializedSize = 0;
    for (const auto &entry : _entries) {
        serializedSize += entry.serializedSize();
    }
    return serializedSize;
}

void DirEntryList::deserializeFrom(const void *data, uint64_t size) {
    _entries.clear();
    const char *pos = static_cast<const char*>(data);
    while (pos < static_cast<const char*>(data) + size) {
        pos = DirEntry::deserializeAndAddToVector(pos, &_entries);
    }
}

bool DirEntryList::_hasChild(const string &name) const {
    auto found = std::find_if(_entries.begin(), _entries.end(), [name] (const DirEntry &entry) {
        return entry.name == name;
    });
    return found != _entries.end();
}

void DirEntryList::add(const string &name, const Key &blobKey, fspp::Dir::EntryType entryType, mode_t mode,
                            uid_t uid, gid_t gid) {
    if (_hasChild(name)) {
        throw fspp::fuse::FuseErrnoException(EEXIST);
    }

    _entries.emplace_back(entryType, name, blobKey, mode, uid, gid);
}

const DirEntry &DirEntryList::get(const string &name) const {
    auto found = std::find_if(_entries.begin(), _entries.end(), [name] (const DirEntry &entry) {
        return entry.name == name;
    });
    if (found == _entries.end()) {
        throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    return *found;
}

const DirEntry &DirEntryList::get(const Key &key) const {
    return *_find(key);
}

void DirEntryList::remove(const Key &key) {
    auto found = _find(key);
    _entries.erase(found);
}

vector<DirEntry>::iterator DirEntryList::_find(const Key &key) {
    auto found = std::find_if(_entries.begin(), _entries.end(), [key] (const DirEntry &entry) {
        return entry.key == key;
    });
    if (found == _entries.end()) {
        throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    return found;
}

vector<DirEntry>::const_iterator DirEntryList::_find(const Key &key) const {
    return const_cast<DirEntryList*>(this)->_find(key);
}

size_t DirEntryList::size() const {
    return _entries.size();
}

DirEntryList::const_iterator DirEntryList::begin() const {
    return _entries.begin();
}

DirEntryList::const_iterator DirEntryList::end() const {
    return _entries.end();
}

void DirEntryList::setMode(const Key &key, mode_t mode) {
    auto found = _find(key);
    ASSERT ((S_ISREG(mode) && S_ISREG(found->mode)) || (S_ISDIR(mode) && S_ISDIR(found->mode)) || (S_ISLNK(mode)), "Unknown mode in entry");
    found->mode = mode;
}

bool DirEntryList::setUidGid(const Key &key, uid_t uid, gid_t gid) {
    auto found = _find(key);
    bool changed = false;
    if (uid != (uid_t)-1) {
        found->uid = uid;
        changed = true;
    }
    if (gid != (gid_t)-1) {
        found->gid = gid;
        changed = true;
    }
    return changed;
}

}
}