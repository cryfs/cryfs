#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FILEBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FILEBLOBREF_H

#include "FsBlobRef.h"
#include "cryfs/impl/filesystem/cachingfsblobstore/FileBlobRef.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class FileBlobRef final: public FsBlobRef {
public:
    FileBlobRef(cachingfsblobstore::FileBlobRef *base) : _base(base) {}

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

    fspp::num_bytes_t lstat_size() const override {
        return _base->lstat_size();
    }

    const blockstore::BlockId &parentPointer() const override {
        return _base->parentPointer();
    }

    void setParentPointer(const blockstore::BlockId &parentId) override {
        return _base->setParentPointer(parentId);
    }

private:
    cachingfsblobstore::FileBlobRef *_base;

    DISALLOW_COPY_AND_ASSIGN(FileBlobRef);
};

}
}

#endif
