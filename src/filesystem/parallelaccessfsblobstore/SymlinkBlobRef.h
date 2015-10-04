#ifndef CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_SYMLINKBLOBREF_H
#define CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_SYMLINKBLOBREF_H

#include "FsBlobRef.h"
#include "../fsblobstore/SymlinkBlob.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class SymlinkBlobRef: public FsBlobRef {
public:
    SymlinkBlobRef(fsblobstore::SymlinkBlob *base) : _base(base) {}

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
    fsblobstore::SymlinkBlob *_base;
};

}
}

#endif
