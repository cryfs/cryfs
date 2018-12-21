#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FSBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FSBLOBREF_H

#include <parallelaccessstore/ParallelAccessStore.h>
#include "cryfs/impl/filesystem/cachingfsblobstore/FsBlobRef.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class FsBlobRef: public parallelaccessstore::ParallelAccessStore<cachingfsblobstore::FsBlobRef, FsBlobRef, blockstore::BlockId>::ResourceRefBase {
public:
    virtual ~FsBlobRef() {}
    virtual const blockstore::BlockId &blockId() const = 0;
    virtual fspp::num_bytes_t lstat_size() const = 0;
    virtual const blockstore::BlockId &parentPointer() const = 0;
    virtual void setParentPointer(const blockstore::BlockId &parentId) = 0;

protected:
    FsBlobRef() {}

private:
    DISALLOW_COPY_AND_ASSIGN(FsBlobRef);
};

}
}

#endif
