#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_DIRBLOBREF_H
#define MESSMER_CRYFS_FILESYSTEM_CACHINGFSBLOBSTORE_DIRBLOBREF_H

#include "FsBlobRef.h"
#include "../fsblobstore/DirBlob.h"

namespace cryfs {
namespace cachingfsblobstore {

class DirBlobRef: public FsBlobRef {
public:
    DirBlobRef(cpputils::unique_ref<fsblobstore::DirBlob> base, CachingFsBlobStore *fsBlobStore):
            FsBlobRef(std::move(base), fsBlobStore),
            _base(dynamic_cast<fsblobstore::DirBlob*>(baseBlob())) {
        ASSERT(_base != nullptr, "We just initialized this with a pointer to DirBlob. Can't be something else now.");
    }

    using Entry = fsblobstore::DirEntry;

    const Entry &GetChild(const std::string &name) const {
        return _base->GetChild(name);
    }

    const Entry &GetChild(const blockstore::Key &key) const {
        return _base->GetChild(key);
    }

    void RemoveChild(const blockstore::Key &key) {
        return _base->RemoveChild(key);
    }

    void flush() {
        return _base->flush();
    }

    void AddChild(const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType type,
                  mode_t mode, uid_t uid, gid_t gid) {
        return _base->AddChild(name, blobKey, type, mode, uid, gid);
    }

    void statChild(const blockstore::Key &key, struct ::stat *result) const {
        return _base->statChild(key, result);
    }

    void chmodChild(const blockstore::Key &key, mode_t mode) {
        return _base->chmodChild(key, mode);
    }

    void chownChild(const blockstore::Key &key, uid_t uid, gid_t gid) {
        return _base->chownChild(key, uid, gid);
    }

    void AddChildDir(const std::string &name, const blockstore::Key &blobKey, mode_t mode, uid_t uid, gid_t gid) {
        return _base->AddChildDir(name, blobKey, mode, uid, gid);
    }

    void AddChildFile(const std::string &name, const blockstore::Key &blobKey, mode_t mode, uid_t uid, gid_t gid) {
        return _base->AddChildFile(name, blobKey, mode, uid, gid);
    }

    void AddChildSymlink(const std::string &name, const blockstore::Key &blobKey, uid_t uid, gid_t gid) {
        return _base->AddChildSymlink(name, blobKey, uid, gid);
    }

    void AppendChildrenTo(std::vector<fspp::Dir::Entry> *result) const {
        return _base->AppendChildrenTo(result);
    }

    const blockstore::Key &key() const {
        return _base->key();
    }

    off_t lstat_size() const {
        return _base->lstat_size();
    }

    void setLstatSizeGetter(std::function<off_t(const blockstore::Key&)> getLstatSize) {
        return _base->setLstatSizeGetter(getLstatSize);
    }

private:

    fsblobstore::DirBlob *_base;

    DISALLOW_COPY_AND_ASSIGN(DirBlobRef);
};

}
}

#endif
