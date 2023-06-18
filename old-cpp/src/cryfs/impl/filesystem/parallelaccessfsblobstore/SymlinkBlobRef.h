#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_SYMLINKBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_SYMLINKBLOBREF_H

#include "FsBlobRef.h"
#include "cryfs/impl/filesystem/cachingfsblobstore/SymlinkBlobRef.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class SymlinkBlobRef final: public FsBlobRef {
public:
    SymlinkBlobRef(cachingfsblobstore::SymlinkBlobRef *base) : _base(base) {}

    const boost::filesystem::path &target() const {
        return _base->target();
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
    cachingfsblobstore::SymlinkBlobRef *_base;

    DISALLOW_COPY_AND_ASSIGN(SymlinkBlobRef);
};

}
}

#endif
