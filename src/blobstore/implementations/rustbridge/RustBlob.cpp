#include "RustBlob.h"
#include "helpers.h"

namespace blobstore
{
    namespace rust
    {
        RustBlob::RustBlob(::rust::Box<bridge::RustBlobBridge> blob)
            : _blob(std::move(blob)), _blobId(helpers::cast_blobid(*_blob->blob_id())) {}
        
        RustBlob::~RustBlob() {
            _blob->async_drop();
        }

        const blockstore::BlockId &RustBlob::blockId() const
        {
            return _blobId;
        }

        uint64_t RustBlob::size() const
        {
            return _blob->num_bytes();
        }

        void RustBlob::resize(uint64_t numBytes)
        {
            return _blob->resize(numBytes);
        }

        cpputils::Data RustBlob::readAll() const
        {
            return helpers::cast_data(&*_blob->read_all());
        }

        void RustBlob::read(void *target, uint64_t offset, uint64_t size) const
        {
            return _blob->read(::rust::Slice<uint8_t>{static_cast<uint8_t *>(target), size}, offset);
        }

        uint64_t RustBlob::tryRead(void *target, uint64_t offset, uint64_t size) const
        {
            return _blob->try_read(::rust::Slice<uint8_t>{static_cast<uint8_t *>(target), size}, offset);
        }

        void RustBlob::write(const void *source, uint64_t offset, uint64_t size)
        {
            return _blob->write(::rust::Slice<const uint8_t>{static_cast<const uint8_t *>(source), size}, offset);
        }

        void RustBlob::flush()
        {
            _blob->flush();
        }

        uint32_t RustBlob::numNodes() const
        {
            return _blob->num_nodes();
        }

        void RustBlob::remove()
        {
            _blob->remove();
        }
    }
}
