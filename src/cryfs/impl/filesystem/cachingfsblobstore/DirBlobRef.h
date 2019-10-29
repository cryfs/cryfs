#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_DIRBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_DIRBLOBREF_H

#include "FsBlobRef.h"
#include "cryfs/impl/filesystem/fsblobstore/DirBlob.h"
#include "cryfs/impl/filesystem/fsblobstore/utils/TimestampUpdateBehavior.h"
#include <fspp/fs_interface/Node.h>

namespace cryfs {
namespace cachingfsblobstore {

class DirBlobRef final: public FsBlobRef {
public:
    DirBlobRef(cpputils::unique_ref<fsblobstore::DirBlob> base, CachingFsBlobStore *fsBlobStore):
            FsBlobRef(std::move(base), fsBlobStore),
            _base(dynamic_cast<fsblobstore::DirBlob*>(baseBlob())) {
        ASSERT(_base != nullptr, "We just initialized this with a pointer to DirBlob. Can't be something else now.");
    }

    using Entry = fsblobstore::DirEntry;

    boost::optional<const Entry&> GetChild(const std::string &name) const {
        return _base->GetChild(name);
    }

    boost::optional<const Entry&> GetChild(const blockstore::BlockId &blockId) const {
        return _base->GetChild(blockId);
    }

    size_t NumChildren() const {
        return _base->NumChildren();
    }

    void RemoveChild(const blockstore::BlockId &blockId) {
        return _base->RemoveChild(blockId);
    }

    void RemoveChild(const std::string &name) {
        return _base->RemoveChild(name);
    }

    void flush() {
        return _base->flush();
    }

    void AddOrOverwriteChild(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::EntryType type,
                  const std::function<void (const fsblobstore::DirEntry &entry)>& onOverwritten) {
        return _base->AddOrOverwriteChild(name, blobId, type, onOverwritten);
    }

    void RenameChild(const blockstore::BlockId &blockId, const std::string &newName, const std::function<void (const fsblobstore::DirEntry &)>& onOverwritten) {
        return _base->RenameChild(blockId, newName, onOverwritten);
    }

    void AddChildDir(const std::string &name, const blockstore::BlockId &blobId) {
        return _base->AddChildDir(name, blobId);
    }

    void AddChildFile(const std::string &name, const blockstore::BlockId &blobId) {
        return _base->AddChildFile(name, blobId);
    }

    void AddChildSymlink(const std::string &name, const blockstore::BlockId &blobId) {
        return _base->AddChildSymlink(name, blobId);
    }

    void AddChildHardlink(const std::string& name, const blockstore::BlockId &blobId, fspp::Dir::EntryType type) {
      return _base->AddChildHardlink(name, blobId, type);
    }

    void AppendChildrenTo(std::vector<fspp::Dir::Entry> *result) const {
        return _base->AppendChildrenTo(result);
    }

    const blockstore::BlockId &blockId() const override {
        return _base->blockId();
    }


    const FsBlobView::Metadata& metaData() {
      return _base->metaData();
    }

    void chown(fspp::uid_t uid, fspp::gid_t gid) {
      return _base->chown(uid, gid);
    }
    void chmod(fspp::mode_t mode) {
      return _base->chmod(mode);
    }

    fspp::stat_info stat() {
      return _base->stat();
    }

    // increase link count by one
    void link() {
      return _base->link();
    }
    // decrease link count by one and return if this was the last link and the node has
    // to be removed. Not that the removal must be done externally;
    bool unlink() {
      return _base->unlink();
    }

    void updateAccessTimestamp() const override {
      return _base->updateAccessTimestamp();
    }

    void updateModificationTimestamp() override {
      return _base->updateModificationTimestamp();
    }
    
    void updateChangeTimestamp() {
      return _base->updateChangeTimestamp();
    }

    void utimens(timespec atime, timespec mtime) override {
      return _base->utimens(atime, mtime);
    }

private:

    fsblobstore::DirBlob *_base;

    DISALLOW_COPY_AND_ASSIGN(DirBlobRef);
};

}
}

#endif
