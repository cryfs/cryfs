#ifndef CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FILEBLOBREF_H
#define CRYFS_FILESYSTEM_PARALLELACCESSFSBLOBSTORE_FILEBLOBREF_H

#include "FsBlobRef.h"
#include "../fsblobstore/FileBlob.h"

namespace cryfs {
namespace parallelaccessfsblobstore {

class FileBlobRef: public FsBlobRef {
public:
    FileBlobRef(fsblobstore::FileBlob *base) : _base(base) {}

    void resize(off_t size) {
        return _base->resize(size);
    }

    off_t size() const {
        return _base->size();
    }

    ssize_t read(void *target, uint64_t offset, uint64_t count) const {
        return _base->read(target, offset, count);
    }

    void write(const void *source, uint64_t offset, uint64_t count) {
        return _base->write(source, offset, count);
    }

    void flush() {
        return _base->flush();
    }

    const blockstore::Key &key() const {
        return _base->key();
    }

    off_t lstat_size() const {
        return _base->lstat_size();
    }

private:
    fsblobstore::FileBlob *_base;
};

}
}

#endif
