#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_DIRBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_DIRBLOBREF_H

#include "FsBlobRef.h"
#include "../cachingfsblobstore/DirBlobRef.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class DirBlobRef final: public FsBlobRef {
public:
    DirBlobRef(cachingfsblobstore::DirBlobRef *base): _base(base) {}

    using Entry = fsblobstore::DirEntry;

    boost::optional<const Entry&> GetChild(const std::string &name) const {
        return _base->GetChild(name);
    }

    boost::optional<const Entry&> GetChild(const blockstore::Key &key) const {
        return _base->GetChild(key);
    }

    size_t NumChildren() const {
        return _base->NumChildren();
    }

    void RemoveChild(const blockstore::Key &key) {
        return _base->RemoveChild(key);
    }

    void RemoveChild(const std::string &name) {
        return _base->RemoveChild(name);
    }

    void flush() {
        return _base->flush();
    }

    void AddOrOverwriteChild(const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType type,
                  mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                  std::function<void (const blockstore::Key &key)> onOverwritten) {
        return _base->AddOrOverwriteChild(name, blobKey, type, mode, uid, gid, lastAccessTime, lastModificationTime, onOverwritten);
    }

    void RenameChild(const blockstore::Key &key, const std::string &newName, std::function<void (const blockstore::Key &key)> onOverwritten) {
        return _base->RenameChild(key, newName, onOverwritten);
    }

    void statChild(const blockstore::Key &key, struct ::stat *result) const {
        return _base->statChild(key, result);
    }

    void statChildExceptSize(const blockstore::Key &key, struct ::stat *result) const {
        return _base->statChildExceptSize(key, result);
    }

    void updateAccessTimestampForChild(const blockstore::Key &key) {
        return _base->updateAccessTimestampForChild(key);
    }

    void updateModificationTimestampForChild(const blockstore::Key &key) {
        return _base->updateModificationTimestampForChild(key);
    }

    void chmodChild(const blockstore::Key &key, mode_t mode) {
        return _base->chmodChild(key, mode);
    }

    void chownChild(const blockstore::Key &key, uid_t uid, gid_t gid) {
        return _base->chownChild(key, uid, gid);
    }

    void utimensChild(const blockstore::Key &key, timespec lastAccessTime, timespec lastModificationTime) {
        return _base->utimensChild(key, lastAccessTime, lastModificationTime);
    }

    void AddChildDir(const std::string &name, const blockstore::Key &blobKey, mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
        return _base->AddChildDir(name, blobKey, mode, uid, gid, lastAccessTime, lastModificationTime);
    }

    void AddChildFile(const std::string &name, const blockstore::Key &blobKey, mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
        return _base->AddChildFile(name, blobKey, mode, uid, gid, lastAccessTime, lastModificationTime);
    }

    void AddChildSymlink(const std::string &name, const blockstore::Key &blobKey, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
        return _base->AddChildSymlink(name, blobKey, uid, gid, lastAccessTime, lastModificationTime);
    }

    void AppendChildrenTo(std::vector<fspp::Dir::Entry> *result) const {
        return _base->AppendChildrenTo(result);
    }

    const blockstore::Key &key() const {
        return _base->key();
    }

    off_t lstat_size() const {
        return _base->lstat_size();
    }

private:
    cachingfsblobstore::DirBlobRef *_base;

    DISALLOW_COPY_AND_ASSIGN(DirBlobRef);
};

}
}

#endif
