#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FSBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FSBLOBREF_H

#include <parallelaccessstore/ParallelAccessStore.h>
#include "../cachingfsblobstore/FsBlobRef.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class FsBlobRef: public parallelaccessstore::ParallelAccessStore<cachingfsblobstore::FsBlobRef, FsBlobRef, blockstore::Key>::ResourceRefBase {
public:
    virtual ~FsBlobRef() {}
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
