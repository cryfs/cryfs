#include "DirEntryList.h"
#include <limits>
#include <cpp-utils/system/time.h>

//TODO Get rid of that in favor of better error handling
#include <fspp/fuse/FuseErrnoException.h>

using cpputils::Data;
using std::string;
using std::vector;
using blockstore::Key;

namespace cryfs {
namespace fsblobstore {

DirEntryList::DirEntryList() : _entries() {
}

Data DirEntryList::serialize() const {
    Data serialized(_serializedSize());
    unsigned int offset = 0;
    for (auto iter = _entries.begin(); iter != _entries.end(); ++iter) {
        ASSERT(iter == _entries.begin() || std::less<Key>()((iter-1)->key(), iter->key()), "Invariant hurt: Directory entries should be ordered by key and not have duplicate keys.");
        iter->serialize(static_cast<uint8_t*>(serialized.dataOffset(offset)));
        offset += iter->serializedSize();
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
        ASSERT(_entries.size() == 1 || std::less<Key>()(_entries[_entries.size()-2].key(), _entries[_entries.size()-1].key()), "Invariant hurt: Directory entries should be ordered by key and not have duplicate keys.");
    }
}

bool DirEntryList::_hasChild(const string &name) const {
    return _entries.end() != _findByName(name);
}

void DirEntryList::add(const string &name, const Key &blobKey, fspp::Dir::EntryType entryType, mode_t mode,
                            uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
    if (_hasChild(name)) {
        throw fspp::fuse::FuseErrnoException(EEXIST);
    }
    _add(name, blobKey, entryType, mode, uid, gid, lastAccessTime, lastModificationTime);
}

void DirEntryList::_add(const string &name, const Key &blobKey, fspp::Dir::EntryType entryType, mode_t mode,
                       uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
    auto insert_pos = _findUpperBound(blobKey);
    _entries.emplace(insert_pos, entryType, name, blobKey, mode, uid, gid, lastAccessTime, lastModificationTime, cpputils::time::now());
}

void DirEntryList::addOrOverwrite(const string &name, const Key &blobKey, fspp::Dir::EntryType entryType, mode_t mode,
                       uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                       std::function<void (const blockstore::Key &key)> onOverwritten) {
    auto found = _findByName(name);
    if (found != _entries.end()) {
        onOverwritten(found->key());
        _overwrite(found, name, blobKey, entryType, mode, uid, gid, lastAccessTime, lastModificationTime);
    } else {
        _add(name, blobKey, entryType, mode, uid, gid, lastAccessTime, lastModificationTime);
    }
}

void DirEntryList::rename(const blockstore::Key &key, const std::string &name, std::function<void (const blockstore::Key &key)> onOverwritten) {
    auto foundSameName = _findByName(name);
    if (foundSameName != _entries.end() && foundSameName->key() != key) {
        _checkAllowedOverwrite(foundSameName->type(), _findByKey(key)->type());
        onOverwritten(foundSameName->key());
        _entries.erase(foundSameName);
    }

    _findByKey(key)->setName(name);
}

void DirEntryList::_checkAllowedOverwrite(fspp::Dir::EntryType oldType, fspp::Dir::EntryType newType) {
    if (oldType != newType) {
        if (oldType == fspp::Dir::EntryType::DIR) {
            // new path is an existing directory, but old path is not a directory
            throw fspp::fuse::FuseErrnoException(EISDIR);
        }
        if (newType == fspp::Dir::EntryType::DIR) {
            // oldpath is a directory, and newpath exists but is not a directory.
            throw fspp::fuse::FuseErrnoException(ENOTDIR);
        }
    }
}

void DirEntryList::_overwrite(vector<DirEntry>::iterator entry, const string &name, const Key &blobKey, fspp::Dir::EntryType entryType, mode_t mode,
                        uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
    _checkAllowedOverwrite(entry->type(), entryType);
    // The new entry has possibly a different key, so it has to be in a different list position (list is ordered by keys).
    // That's why we remove-and-add instead of just modifying the existing entry.
    _entries.erase(entry);
    _add(name, blobKey, entryType, mode, uid, gid, lastAccessTime, lastModificationTime);
}

boost::optional<const DirEntry&> DirEntryList::get(const string &name) const {
    auto found = _findByName(name);
    if (found == _entries.end()) {
        return boost::none;
    }
    return *found;
}

boost::optional<const DirEntry&> DirEntryList::get(const Key &key) const {
    auto found = _findByKey(key);
    if (found == _entries.end()) {
        return boost::none;
    }
    return *found;
}

void DirEntryList::remove(const string &name) {
    auto found = _findByName(name);
    if (found == _entries.end()) {
        throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    _entries.erase(found);
}

void DirEntryList::remove(const Key &key) {
    auto lowerBound = _findLowerBound(key);
    auto upperBound = std::find_if(lowerBound, _entries.end(), [&key] (const DirEntry &entry) {
        return entry.key() != key;
    });
    _entries.erase(lowerBound, upperBound);
}

vector<DirEntry>::iterator DirEntryList::_findByName(const string &name) {
    return std::find_if(_entries.begin(), _entries.end(), [&name] (const DirEntry &entry) {
        return entry.name() == name;
    });
}

vector<DirEntry>::const_iterator DirEntryList::_findByName(const string &name) const {
    return const_cast<DirEntryList*>(this)->_findByName(name);
}

vector<DirEntry>::iterator DirEntryList::_findByKey(const Key &key) {
    auto found = _findLowerBound(key);
    if (found == _entries.end() || found->key() != key) {
        throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    return found;
}

vector<DirEntry>::iterator DirEntryList::_findLowerBound(const Key &key) {
    return _findFirst(key, [&key] (const DirEntry &entry) {
        return !std::less<Key>()(entry.key(), key);
    });
}

vector<DirEntry>::iterator DirEntryList::_findUpperBound(const Key &key) {
    return _findFirst(key, [&key] (const DirEntry &entry) {
        return std::less<Key>()(key, entry.key());
    });
}

vector<DirEntry>::iterator DirEntryList::_findFirst(const Key &hint, std::function<bool (const DirEntry&)> pred) {
    //TODO Factor out a datastructure that keeps a sorted std::vector and allows these _findLowerBound()/_findUpperBound operations using this hinted linear search
    if (_entries.size() == 0) {
        return _entries.end();
    }
    double startpos_percent = static_cast<double>(*static_cast<const unsigned char*>(hint.data())) / std::numeric_limits<unsigned char>::max();
    auto iter = _entries.begin() + static_cast<int>(startpos_percent * (_entries.size()-1));
    ASSERT(iter >= _entries.begin() && iter < _entries.end(), "Startpos out of range");
    while(iter != _entries.begin() && pred(*iter)) {
        --iter;
    }
    while(iter != _entries.end() && !pred(*iter)) {
        ++iter;
    }
    return iter;
}

vector<DirEntry>::const_iterator DirEntryList::_findByKey(const Key &key) const {
    return const_cast<DirEntryList*>(this)->_findByKey(key);
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
    auto found = _findByKey(key);
    ASSERT ((S_ISREG(mode) && S_ISREG(found->mode())) || (S_ISDIR(mode) && S_ISDIR(found->mode())) || (S_ISLNK(mode)), "Unknown mode in entry");
    found->setMode(mode);
}

bool DirEntryList::setUidGid(const Key &key, uid_t uid, gid_t gid) {
    auto found = _findByKey(key);
    bool changed = false;
    if (uid != (uid_t)-1) {
        found->setUid(uid);
        changed = true;
    }
    if (gid != (gid_t)-1) {
        found->setGid(gid);
        changed = true;
    }
    return changed;
}

void DirEntryList::setAccessTimes(const blockstore::Key &key, timespec lastAccessTime, timespec lastModificationTime) {
    auto found = _findByKey(key);
    found->setLastAccessTime(lastAccessTime);
    found->setLastModificationTime(lastModificationTime);
}

void DirEntryList::updateAccessTimestampForChild(const blockstore::Key &key) {
    auto found = _findByKey(key);
    // TODO Think about implementing relatime behavior. Currently, CryFS follows strictatime.
    found->setLastAccessTime(cpputils::time::now());
}

void DirEntryList::updateModificationTimestampForChild(const blockstore::Key &key) {
    auto found = _findByKey(key);
    found->setLastModificationTime(cpputils::time::now());
}

}
}
