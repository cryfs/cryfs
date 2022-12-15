#include "RustFileBlob.h"
#include <cryfs/impl/filesystem/rustfsblobstore/helpers.h>

using blockstore::BlockId;
using cryfs::fsblobstore::rust::helpers::cast_blobid;

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            RustFileBlob::RustFileBlob(::rust::Box<bridge::RustFileBlobBridge> fileBlob)
                : _fileBlob(std::move(fileBlob))
            {
            }

            fspp::num_bytes_t RustFileBlob::read(void *target, fspp::num_bytes_t offset, fspp::num_bytes_t count)
            {
                return fspp::num_bytes_t(
                    _fileBlob->try_read(::rust::Slice<uint8_t>(reinterpret_cast<uint8_t *>(target), count.value()), offset.value()));
            }

            void RustFileBlob::write(const void *source, fspp::num_bytes_t offset, fspp::num_bytes_t count)
            {
                _fileBlob->write(::rust::Slice<const uint8_t>(reinterpret_cast<const uint8_t *>(source), count.value()), offset.value());
            }

            void RustFileBlob::flush()
            {
                _fileBlob->flush();
            }

            void RustFileBlob::resize(fspp::num_bytes_t size)
            {
                _fileBlob->resize(size.value());
            }

            fspp::num_bytes_t RustFileBlob::size()
            {
                return fspp::num_bytes_t(_fileBlob->num_bytes());
            }

            BlockId RustFileBlob::parent() const
            {
                return cast_blobid(*_fileBlob->parent());
            }

            BlockId RustFileBlob::blockId() const
            {
                return cast_blobid(*_fileBlob->blob_id());
            }
        }
    }
}