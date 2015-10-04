#ifndef CRYFS_PARALLELACCESSFSBLOBSTORE_FSBLOBREF_H
#define CRYFS_PARALLELACCESSFSBLOBSTORE_FSBLOBREF_H

#include <messmer/parallelaccessstore/ParallelAccessStore.h>
#include "../fsblobstore/FsBlob.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class FsBlobRef: public parallelaccessstore::ParallelAccessStore<fsblobstore::FsBlob, FsBlobRef, blockstore::Key>::ResourceRefBase {
public:
    virtual const blockstore::Key &key() const = 0;
    virtual off_t lstat_size() const = 0;

protected:
    FsBlobRef() {}

private:
    DISALLOW_COPY_AND_ASSIGN(FsBlobRef);
};

}
}

#endif
