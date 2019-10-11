#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FILEBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FILEBLOBREF_H

#include "FsBlobRef.h"
#include "cryfs/impl/filesystem/cachingfsblobstore/FileBlobRef.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class FileBlobRef final: public FsBlobRef {
public:
    explicit FileBlobRef(cachingfsblobstore::FileBlobRef *base) : _base(base) {}

    void resize(fspp::num_bytes_t size) {
        return _base->resize(size);
    }

    fspp::num_bytes_t size() const {
        return _base->size();
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

    void utimens(timespec atime, timespec mtime) override {
      return _base->utimens(atime, mtime);
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

    void updateAccessTimestamp() const override {
      return _base->updateAccessTimestamp();
    }

    void updateModificationTimestamp() override {
      return _base->updateModificationTimestamp();
    }

    void updateChangeTimestamp() override {
      return _base->updateChangeTimestamp();
    }

    fspp::num_bytes_t read(void *target, fspp::num_bytes_t offset, fspp::num_bytes_t count) const {
        return _base->read(target, offset, count);
    }

    void write(const void *source, fspp::num_bytes_t offset, fspp::num_bytes_t count) {
        return _base->write(source, offset, count);
    }

    void flush() {
        return _base->flush();
    }

    const blockstore::BlockId &blockId() const override {
        return _base->blockId();
    }

private:
    cachingfsblobstore::FileBlobRef *_base;

    DISALLOW_COPY_AND_ASSIGN(FileBlobRef);
};

}
}

#endif
