#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_FILEBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_FILEBLOBREF_H

#include "FsBlobRef.h"
#include "../fsblobstore/FileBlob.h"

namespace cryfs {
namespace cachingfsblobstore {

class FileBlobRef final: public FsBlobRef {
public:
    FileBlobRef(cpputils::unique_ref<fsblobstore::FileBlob> base, CachingFsBlobStore *fsBlobStore)
            :FsBlobRef(std::move(base), fsBlobStore),
            _base(dynamic_cast<fsblobstore::FileBlob*>(baseBlob())) {
        ASSERT(_base != nullptr, "We just initialized this with a pointer to FileBlob. Can't be something else now.");
    }

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

    DISALLOW_COPY_AND_ASSIGN(FileBlobRef);
};

}
}

#endif
