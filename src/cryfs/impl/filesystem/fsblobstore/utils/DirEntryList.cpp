#include "DirEntryList.h"
#include <limits>
#include <cpp-utils/system/time.h>

//TODO Get rid of that in favor of better error handling
#include <fspp/fs_interface/FuseErrnoException.h>

using cpputils::Data;
using std::string;
using std::vector;
using blockstore::BlockId;

namespace cryfs {
namespace fsblobstore {

DirEntryList::DirEntryList() : _entries() {
}

Data DirEntryList::serialize() const {
    return serializeExternal(_entries);
}

cpputils::Data DirEntryList::serializeExternal(const std::vector<DirEntry>& entries) {
  Data serialized(_serializedSizeExternal(entries));
  unsigned int offset = 0;
  for (auto iter = entries.begin(); iter != entries.end(); ++iter) {

    ASSERT(iter == entries.begin() || std::less_equal<BlockId>()((iter-1)->blockId(), iter->blockId()), "Invariant hurt: Directory entries should be ordered by blockId. Duplicates are allowed in the case of hard links");
    iter->serialize(static_cast<uint8_t*>(serialized.dataOffset(offset)));
    offset += iter->serializedSize();
  }
  return serialized;

}

uint64_t DirEntryList::_serializedSize() const {
  return _serializedSizeExternal(_entries);
}

uint64_t DirEntryList::_serializedSizeExternal(const std::vector<DirEntry>& entries) {
  uint64_t serializedSize = 0;
  for (const auto &entry : entries) {
    serializedSize += entry.serializedSize();
  }
  return serializedSize;
}

void DirEntryList::deserializeFrom(const void *data, uint64_t size) {
    _entries.clear();
    const char *pos = static_cast<const char*>(data);
    while (pos < static_cast<const char*>(data) + size) {
        pos = DirEntry::deserializeAndAddToVector(pos, &_entries);
        ASSERT(_entries.size() == 1 || std::less_equal<BlockId>()(_entries[_entries.size()-2].blockId(), _entries[_entries.size()-1].blockId()), "Invariant hurt: Directory entries should be ordered by blockId. Duplicates are allowed in the case of hard links");
    }
}

bool DirEntryList::_hasChild(const string &name) const {
    return _entries.end() != _findByName(name);
}

void DirEntryList::add(const string &name, const BlockId &blobId, fspp::Dir::NodeType entryType) {
    if (_hasChild(name)) {
        throw fspp::fuse::FuseErrnoException(EEXIST);
    }
    _add(name, blobId, entryType);
}

void DirEntryList::_add(const string &name, const BlockId &blobId, fspp::Dir::NodeType entryType) {
    //auto insert_pos = _findUpperBound(blobId);
      auto insert_pos = std::upper_bound(_entries.begin(), _entries.end(), blobId, [](const BlockId& value, const DirEntry& entry){return std::less<BlockId>()(value, entry.blockId());});
      _entries.emplace(insert_pos, entryType, name, blobId);
  }

  DirEntryList::AddOver DirEntryList::addOrOverwrite(const string &name, const BlockId &blobId, fspp::Dir::NodeType entryType,
                         const std::function<void (const DirEntry &entry)>& onOverwritten) {
      auto found = _findByName(name);
      if (found != _entries.end()) {
          onOverwritten(*found);
          _overwrite(found, name, blobId, entryType);
          return AddOver::OVERWRITE;
      } else {
          _add(name, blobId, entryType);
          return AddOver::ADD;
      }
  }

  void DirEntryList::rename(const blockstore::BlockId &blockId, const std::string &name, const std::function<void (const DirEntry &entry)>& onOverwritten) {
      auto foundSameName = _findByName(name);
      if (foundSameName != _entries.end() && foundSameName->blockId() != blockId) {
        _checkAllowedOverwrite(foundSameName->type(), _findById(blockId)->type());
        onOverwritten(*foundSameName);
        _entries.erase(foundSameName);
    }

    _findById(blockId)->setName(name);
}

void DirEntryList::_checkAllowedOverwrite(fspp::Dir::NodeType oldType, fspp::Dir::NodeType newType) {
    if (oldType != newType) {
        if (oldType == fspp::Dir::NodeType::DIR) {
            // new path is an existing directory, but old path is not a directory
            throw fspp::fuse::FuseErrnoException(EISDIR);
        }
        if (newType == fspp::Dir::NodeType::DIR) {
            // oldpath is a directory, and newpath exists but is not a directory.
            throw fspp::fuse::FuseErrnoException(ENOTDIR);
        }
    }
}

void DirEntryList::_overwrite(vector<DirEntry>::iterator entry, const string &name, const BlockId &blobId, fspp::Dir::NodeType entryType) {
    _checkAllowedOverwrite(entry->type(), entryType);
    // The new entry has possibly a different blockId, so it has to be in a different list position (list is ordered by blockIds).
    // That's why we remove-and-add instead of just modifying the existing entry.
    _entries.erase(entry);
    _add(name, blobId, entryType);
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
        throw fspp::fuse::FuseErrnoException(ENOENT);
    }
    return found;
}

vector<DirEntry>::iterator DirEntryList::_findLowerBound(const BlockId &blockId) {
    return _findFirst(blockId, [&blockId] (const DirEntry &entry) {
        return !std::less<BlockId>()(entry.blockId(), blockId);
    });
}

vector<DirEntry>::iterator DirEntryList::_findFirst(const BlockId &hint, const std::function<bool (const DirEntry&)>& pred) {
    //TODO Factor out a datastructure that keeps a sorted std::vector and allows these _findLowerBound()/_findUpperBound operations using this hinted linear search
    if (_entries.empty()) {
        return _entries.end();
    }
    double startpos_percent = static_cast<double>(*static_cast<const unsigned char*>(hint.data().data())) / std::numeric_limits<unsigned char>::max();
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

}
}
