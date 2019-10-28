#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_SYMLINKBLOB_H_
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_SYMLINKBLOB_H_

#include <boost/filesystem/path.hpp>
#include "FsBlob.h"

namespace cryfs {
    namespace fsblobstore {

        class SymlinkBlob final: public FsBlob {
        public:
            static cpputils::unique_ref<SymlinkBlob> InitializeSymlink(cpputils::unique_ref<blobstore::Blob> blob,
                                                                       const boost::filesystem::path &target, const FsBlobView::Metadata &meta, const TimestampUpdateBehavior& behavior);

            SymlinkBlob(cpputils::unique_ref<blobstore::Blob> blob, const TimestampUpdateBehavior& behavior);

            const boost::filesystem::path &target() const;

            void utimens(timespec atime, timespec mtime) {
              return baseBlob().utimens(atime, mtime);
            }

        private:
            boost::filesystem::path _target;

            static boost::filesystem::path _readTargetFromBlob(const FsBlobView &blob);

            DISALLOW_COPY_AND_ASSIGN(SymlinkBlob);
        };
    }
}

#endif
