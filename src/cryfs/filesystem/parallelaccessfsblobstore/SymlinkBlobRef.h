#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_SYMLINKBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_SYMLINKBLOBREF_H

#include "FsBlobRef.h"
#include "../cachingfsblobstore/SymlinkBlobRef.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class SymlinkBlobRef final: public FsBlobRef {
public:
    SymlinkBlobRef(cachingfsblobstore::SymlinkBlobRef *base) : _base(base) {}

    const boost::filesystem::path &target() const {
        return _base->target();
    }

    const blockstore::Key &key() const override {
        return _base->key();
    }

    off_t lstat_size() const override {
        return _base->lstat_size();
    }

    const blockstore::Key &parentPointer() const override {
        return _base->parentPointer();
    }

    void setParentPointer(const blockstore::Key &parentKey) override {
        return _base->setParentPointer(parentKey);
    }

private:
    cachingfsblobstore::SymlinkBlobRef *_base;

    DISALLOW_COPY_AND_ASSIGN(SymlinkBlobRef);
};

}
}

#endif
