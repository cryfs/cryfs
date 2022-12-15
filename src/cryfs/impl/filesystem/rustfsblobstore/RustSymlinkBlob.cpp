#include "RustSymlinkBlob.h"
#include <cryfs/impl/filesystem/rustfsblobstore/helpers.h>

using cryfs::fsblobstore::rust::helpers::cast_blobid;
namespace bf = boost::filesystem;
using blockstore::BlockId;

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            RustSymlinkBlob::RustSymlinkBlob(::rust::Box<bridge::RustSymlinkBlobBridge> symlinkBlob)
                : _symlinkBlob(std::move(symlinkBlob))
            {
            }

            const bf::path RustSymlinkBlob::target()
            {
                return bf::path(_symlinkBlob->target().c_str());
            }

            BlockId RustSymlinkBlob::parent() const
            {
                return cast_blobid(*_symlinkBlob->parent());
            }

            BlockId RustSymlinkBlob::blockId() const
            {
                return cast_blobid(*_symlinkBlob->blob_id());
            }
        }
    }
}