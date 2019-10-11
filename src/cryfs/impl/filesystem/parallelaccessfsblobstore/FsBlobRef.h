#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FSBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FSBLOBREF_H

#include <parallelaccessstore/ParallelAccessStore.h>
#include "cryfs/impl/filesystem/cachingfsblobstore/FsBlobRef.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class FsBlobRef: public parallelaccessstore::ParallelAccessStore<cachingfsblobstore::FsBlobRef, FsBlobRef, blockstore::BlockId>::ResourceRefBase {
public:
    virtual ~FsBlobRef() = default;
    virtual const blockstore::BlockId &blockId() const = 0;
    virtual const FsBlobView::Metadata& metaData() = 0;
    virtual void updateAccessTimestamp() const = 0;
    virtual void updateModificationTimestamp() = 0;
    virtual void updateChangeTimestamp() = 0;
    virtual void chown(fspp::uid_t uid, fspp::gid_t gid) = 0;
    virtual void chmod(fspp::mode_t mode) = 0;
    virtual void utimens(timespec atime, timespec mtime) = 0;

    // increase link count by one
    virtual void link() = 0;
    // decrease link count by one and return if this was the last link and the node has
    // to be removed. Not that the removal must be done externally;
    virtual bool unlink() = 0;

    virtual fspp::stat_info stat() = 0;

protected:
    FsBlobRef() = default;

private:
    DISALLOW_COPY_AND_ASSIGN(FsBlobRef);
};

}
}

#endif
