#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_SYMLINKBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_SYMLINKBLOBREF_H

#include "FsBlobRef.h"
#include "cryfs/impl/filesystem/fsblobstore/SymlinkBlob.h"

namespace cryfs {
namespace cachingfsblobstore {

class SymlinkBlobRef final: public FsBlobRef {
public:
    SymlinkBlobRef(cpputils::unique_ref<fsblobstore::SymlinkBlob> base, CachingFsBlobStore *fsBlobStore)
        :FsBlobRef(std::move(base), fsBlobStore),
        _base(dynamic_cast<fsblobstore::SymlinkBlob*>(baseBlob())) {
        ASSERT(_base != nullptr, "We just initialized this with a pointer to SymlinkBlob. Can't be something else now.");
    }

    const boost::filesystem::path &target() const {
        return _base->target();
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

    void utimens(timespec atime, timespec mtime) override {
      return _base->utimens(atime, mtime);
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

private:

    fsblobstore::SymlinkBlob *_base;

    DISALLOW_COPY_AND_ASSIGN(SymlinkBlobRef);
};

}
}

#endif
