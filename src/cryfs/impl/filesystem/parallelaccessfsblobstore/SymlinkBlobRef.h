#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_SYMLINKBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_SYMLINKBLOBREF_H

#include "FsBlobRef.h"
#include "cryfs/impl/filesystem/cachingfsblobstore/SymlinkBlobRef.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class SymlinkBlobRef final: public FsBlobRef {
public:
    explicit SymlinkBlobRef(cachingfsblobstore::SymlinkBlobRef *base) : _base(base) {}

    const boost::filesystem::path &target() const {
        return _base->target();
    }

    const blockstore::BlockId &blockId() const override {
        return _base->blockId();
    }

    const FsBlobView::Metadata& metaData() override {
      return _base->metaData();
    }

    void chown(fspp::uid_t uid, fspp::gid_t gid) override {
      return _base->chown(uid, gid);
    }
    void chmod(fspp::mode_t mode) override {
      return _base->chmod(mode);
    }

    fspp::stat_info stat() override {
      return _base->stat();
    }

    // increase link count by one
    void link() override {
      return _base->link();
    }
    // decrease link count by one and return if this was the last link and the node has
    // to be removed. Not that the removal must be done externally;
    bool unlink() override {
      return _base->unlink();
    }

    void utimens(timespec atime, timespec mtime) override {
      return _base->utimens(atime, mtime);
    }

    void updateAccessTimestamp() const override {
      return _base->updateAccessTimestamp();
    }

    void updateModificationTimestamp() override {
      return _base->updateModificationTimestamp();
    }

    void updateChangeTimestamp() override {
      return _base->updateChangeTimestamp();
    }

private:
    cachingfsblobstore::SymlinkBlobRef *_base;

    DISALLOW_COPY_AND_ASSIGN(SymlinkBlobRef);
};

}
}

#endif
