#include "RustBlobStore.h"
#include "RustBlob.h"
#include "helpers.h"
#include <cpp-utils/pointer/cast.h>
#include <cpp-utils/assert/assert.h>

using boost::none;
using cpputils::dynamic_pointer_move;

namespace blobstore
{
    namespace rust
    {
        namespace
        {
            cpputils::unique_ref<Blob> cast_blob(::rust::Box<::blobstore::rust::bridge::RustBlobBridge> blob)
            {
                cpputils::unique_ref<Blob> result = cpputils::make_unique_ref<RustBlob>(std::move(blob));
                return result;
            }

            boost::optional<cpputils::unique_ref<Blob>> cast_optional_blob(::rust::Box<::blobstore::rust::bridge::OptionRustBlobBridge> optionBlob)
            {
                if (optionBlob->has_value())
                {
                    // TODO defer to cast_blob
                    auto data = optionBlob->extract_value();
                    cpputils::unique_ref<Blob> result = cpputils::make_unique_ref<RustBlob>(std::move(data));
                    return result;
                }
                else
                {
                    return boost::none;
                }
            }
        }

        RustBlobStore::RustBlobStore(::rust::Box<bridge::RustBlobStoreBridge> blobStore)
            : _blobStore(std::move(blobStore)) {}

        RustBlobStore::~RustBlobStore()
        {
            _blobStore->async_drop();
        }

        cpputils::unique_ref<Blob> RustBlobStore::create()
        {
            return cast_blob(_blobStore->create());
        }

        boost::optional<cpputils::unique_ref<Blob>> RustBlobStore::load(const blockstore::BlockId &blobId)
        {
            return cast_optional_blob(_blobStore->load(*helpers::cast_blobid(blobId)));
        }

        void RustBlobStore::remove(cpputils::unique_ref<Blob> blob)
        {
            auto _blob = dynamic_pointer_move<RustBlob>(blob);
            ASSERT(_blob != none, "Passed Blob in RustBlobStore::remove() is not a BlobOnBlocks.");
            (*_blob)->remove();
        }

        void RustBlobStore::remove(const blockstore::BlockId &blockId)
        {
            _blobStore->remove_by_id(*helpers::cast_blobid(blockId));
        }

        uint64_t RustBlobStore::numBlocks() const
        {
            return _blobStore->num_nodes();
        }

        uint64_t RustBlobStore::estimateSpaceForNumBlocksLeft() const
        {
            return _blobStore->estimate_space_for_num_blocks_left();
        }

        uint64_t RustBlobStore::virtualBlocksizeBytes() const
        {
            return _blobStore->virtual_block_size_bytes();
        }

    }
}
