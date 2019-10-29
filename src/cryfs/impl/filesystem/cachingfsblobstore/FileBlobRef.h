#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_FILEBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_FILEBLOBREF_H

#include "FsBlobRef.h"
#include "cryfs/impl/filesystem/fsblobstore/FileBlob.h"

namespace cryfs {
namespace cachingfsblobstore {

class FileBlobRef final: public FsBlobRef {
public:
    FileBlobRef(cpputils::unique_ref<fsblobstore::FileBlob> base, CachingFsBlobStore *fsBlobStore)
            :FsBlobRef(std::move(base), fsBlobStore),
            _base(dynamic_cast<fsblobstore::FileBlob*>(baseBlob())) {
        ASSERT(_base != nullptr, "We just initialized this with a pointer to FileBlob. Can't be something else now.");
    }

    void resize(fspp::num_bytes_t size) {
        return _base->resize(size);
    }

    fspp::num_bytes_t size() const {
        return _base->size();
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

    fsblobstore::FileBlob *_base;

    DISALLOW_COPY_AND_ASSIGN(FileBlobRef);
};

}
}

#endif
