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

    const blockstore::Key &key() const {
        return _base->key();
    }

    off_t lstat_size() const {
        return _base->lstat_size();
    }

private:
    cachingfsblobstore::SymlinkBlobRef *_base;

    DISALLOW_COPY_AND_ASSIGN(SymlinkBlobRef);
};

}
}

#endif
