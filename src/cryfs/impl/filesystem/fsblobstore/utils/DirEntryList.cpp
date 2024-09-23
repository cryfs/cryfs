#include "DirEntryList.h"
#include "blockstore/utils/BlockId.h"
#include "cpp-utils/assert/assert.h"
#include "cpp-utils/data/Data.h"
#include "cryfs/impl/filesystem/fsblobstore/utils/DirEntry.h"
#include "fspp/fs_interface/Context.h"
#include "fspp/fs_interface/Dir.h"
#include "fspp/fs_interface/Types.h"
#include <algorithm>
#include <boost/none.hpp>
#include <boost/optional/detail/optional_reference_spec.hpp>
#include <cerrno>
#include <cpp-utils/system/time.h>
#include <cstddef>
#include <cstdint>
#include <ctime>
#include <functional>
#include <limits>

//TODO Get rid of that in favor of better error handling
#include <fspp/fs_interface/FuseErrnoException.h>
#include <stdexcept>
#include <string>

using cpputils::Data;
using std::string;
using std::vector;
using blockstore::BlockId;

namespace cryfs {
namespace fsblobstore {

DirEntryList::DirEntryList() : _entries() {
}

Data DirEntryList::serialize() const {
    Data serialized(_serializedSize());
    unsigned int offset = 0;
    for (auto iter = _entries.begin(); iter != _entries.end(); ++iter) {
        ASSERT(iter == _entries.begin() || std::less<BlockId>()((iter-1)->blockId(), iter->blockId()), "Invariant hurt: Directory entries should be ordered by blockId and not have duplicate blockIds.");
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
        ASSERT(_entries.size() == 1 || std::less<BlockId>()(_entries[_entries.size()-2].blockId(), _entries[_entries.size()-1].blockId()), "Invariant hurt: Directory entries should be ordered by blockId and not have duplicate blockIds.");
    }
}

bool DirEntryList::_hasChild(const string &name) const {
    return _entries.end() != _findByName(name);
}

void DirEntryList::add(const string &name, const BlockId &blobId, fspp::Dir::EntryType entryType, fspp::mode_t mode,
                            fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
    if (_hasChild(name)) {
        throw fspp::fuse::FuseErrnoException(EEXIST);
    }
    _add(name, blobId, entryType, mode, uid, gid, lastAccessTime, lastModificationTime);
}

void DirEntryList::_add(const string &name, const BlockId &blobId, fspp::Dir::EntryType entryType, fspp::mode_t mode,
                       fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
    auto insert_pos = _findUpperBound(blobId);
    _entries.emplace(insert_pos, entryType, name, blobId, mode, uid, gid, lastAccessTime, lastModificationTime, cpputils::time::now());
}

void DirEntryList::addOrOverwrite(const string &name, const BlockId &blobId, fspp::Dir::EntryType entryType, fspp::mode_t mode,
                       fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                       std::function<void (const blockstore::BlockId &blockId)> onOverwritten) {
    auto found = _findByName(name);
    if (found != _entries.end()) {
        onOverwritten(found->blockId());
        _overwrite(found, name, blobId, entryType, mode, uid, gid, lastAccessTime, lastModificationTime);
    } else {
        _add(name, blobId, entryType, mode, uid, gid, lastAccessTime, lastModificationTime);
    }
}

void DirEntryList::rename(const blockstore::BlockId &blockId, const std::string &name, std::function<void (const blockstore::BlockId &blockId)> onOverwritten) {
    auto foundSameName = _findByName(name);
    if (foundSameName != _entries.end() && foundSameName->blockId() != blockId) {
        auto found = _findById(blockId);
        if (found == _entries.end()) {
            throw fspp::fuse::FuseErrnoException(ENOENT);
        }
        _checkAllowedOverwrite(foundSameName->type(), found->type());
        onOverwritten(foundSameName->blockId());
        _entries.erase(foundSameName);
    }

    auto found = _findById(blockId);
    if (found == _entries.end()) {
        throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    found->setName(name);
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

void DirEntryList::_overwrite(vector<DirEntry>::iterator entry, const string &name, const BlockId &blobId, fspp::Dir::EntryType entryType, fspp::mode_t mode,
                        fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
    _checkAllowedOverwrite(entry->type(), entryType);
    // The new entry has possibly a different blockId, so it has to be in a different list position (list is ordered by blockIds).
    // That's why we remove-and-add instead of just modifying the existing entry.
    _entries.erase(entry);
    _add(name, blobId, entryType, mode, uid, gid, lastAccessTime, lastModificationTime);
}

boost::optional<const DirEntry&> DirEntryList::get(const string &name) const {
    auto found = _findByName(name);
    if (found == _entries.end()) {
        return boost::none;
    }
    return *found;
}

boost::optional<const DirEntry&> DirEntryList::get(const BlockId &blockId) const {
    auto found = _findById(blockId);
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

void DirEntryList::remove(const BlockId &blockId) {
    auto lowerBound = _findLowerBound(blockId);
    auto upperBound = std::find_if(lowerBound, _entries.end(), [&blockId] (const DirEntry &entry) {
        return entry.blockId() != blockId;
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

vector<DirEntry>::iterator DirEntryList::_findById(const BlockId &blockId) {
    auto found = _findLowerBound(blockId);
    if (found == _entries.end() || found->blockId() != blockId) {
        return _entries.end();
    }
    return found;
}

vector<DirEntry>::iterator DirEntryList::_findLowerBound(const BlockId &blockId) {
    return _findFirst(blockId, [&blockId] (const DirEntry &entry) {
        return !std::less<BlockId>()(entry.blockId(), blockId);
    });
}

vector<DirEntry>::iterator DirEntryList::_findUpperBound(const BlockId &blockId) {
    return _findFirst(blockId, [&blockId] (const DirEntry &entry) {
        return std::less<BlockId>()(blockId, entry.blockId());
    });
}

vector<DirEntry>::iterator DirEntryList::_findFirst(const BlockId &hint, std::function<bool (const DirEntry&)> pred) {
    //TODO Factor out a datastructure that keeps a sorted std::vector and allows these _findLowerBound()/_findUpperBound operations using this hinted linear search
    if (_entries.size() == 0) {
        return _entries.end();
    }
    const double startpos_percent = static_cast<double>(*static_cast<const unsigned char*>(hint.data().data())) / std::numeric_limits<unsigned char>::max();
    auto iter = _entries.begin() + static_cast<int>(startpos_percent * static_cast<double>(_entries.size()-1));
    ASSERT(iter >= _entries.begin() && iter < _entries.end(), "Startpos out of range");
    while(iter != _entries.begin() && pred(*iter)) {
        --iter;
    }
    while(iter != _entries.end() && !pred(*iter)) {
        ++iter;
    }
    return iter;
}

vector<DirEntry>::const_iterator DirEntryList::_findById(const BlockId &blockId) const {
    return const_cast<DirEntryList*>(this)->_findById(blockId);
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

void DirEntryList::setMode(const BlockId &blockId, fspp::mode_t mode) {
    auto found = _findById(blockId);
    if (found == _entries.end()) {
        throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    ASSERT ((mode.hasFileFlag() && found->mode().hasFileFlag()) || (mode.hasDirFlag() && found->mode().hasDirFlag()) || (mode.hasSymlinkFlag()), "Unknown mode in entry");
    found->setMode(mode);
}

bool DirEntryList::setUidGid(const BlockId &blockId, fspp::uid_t uid, fspp::gid_t gid) {
    auto found = _findById(blockId);
    if (found == _entries.end()) {
        throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    bool changed = false;
    if (uid != fspp::uid_t(-1)) {
        found->setUid(uid);
        changed = true;
    }
    if (gid != fspp::gid_t(-1)) {
        found->setGid(gid);
        changed = true;
    }
    return changed;
}

void DirEntryList::setAccessTimes(const blockstore::BlockId &blockId, timespec lastAccessTime, timespec lastModificationTime) {
    auto found = _findById(blockId);
    if (found == _entries.end()) {
        throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    found->setLastAccessTime(lastAccessTime);
    found->setLastModificationTime(lastModificationTime);
}

bool DirEntryList::updateAccessTimestampForChild(const blockstore::BlockId &blockId, fspp::TimestampUpdateBehavior timestampUpdateBehavior) {
    auto found = _findById(blockId);
    if (found == _entries.end()) {
        throw fspp::fuse::FuseErrnoException(ENOENT);
    }

    const timespec lastAccessTime = found->lastAccessTime();
    const timespec lastModificationTime = found->lastModificationTime();
    const timespec now = cpputils::time::now();

    switch (found->type()) {
        case fspp::Dir::EntryType::FILE:
            // fallthrough
        case fspp::Dir::EntryType::SYMLINK:
            if (timestampUpdateBehavior->shouldUpdateATimeOnFileRead(lastAccessTime, lastModificationTime, now)) {
                found->setLastAccessTime(now);
                return true;
            }
            return false;
        case fspp::Dir::EntryType::DIR:
            if (timestampUpdateBehavior->shouldUpdateATimeOnDirectoryRead(lastAccessTime, lastModificationTime, now)) {
                found->setLastAccessTime(now);
                return true;
            }
            return false;
    }
    throw std::logic_error("Unhandled case");
}

void DirEntryList::updateModificationTimestampForChild(const blockstore::BlockId &blockId) {
    auto found = _findById(blockId);
    if (found == _entries.end()) {
        throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    found->setLastModificationTime(cpputils::time::now());
}

}
}
