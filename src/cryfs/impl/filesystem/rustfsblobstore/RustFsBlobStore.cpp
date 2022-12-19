#include "RustFsBlobStore.h"
#include <cryfs/impl/filesystem/rustfsblobstore/helpers.h>

using blockstore::BlockId;
using boost::optional;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cryfs::fsblobstore::rust::helpers::cast_blobid;

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            RustFsBlobStore::RustFsBlobStore(::rust::Box<bridge::RustFsBlobStoreBridge> fsBlobStore)
                : _fsBlobStore(std::move(fsBlobStore)) {}

            RustFsBlobStore::~RustFsBlobStore()
            {
                _fsBlobStore->async_drop();
            }

            unique_ref<RustDirBlob> RustFsBlobStore::createDirBlob(const BlockId &parent)
            {
                return make_unique_ref<RustDirBlob>(_fsBlobStore->create_dir_blob(*cast_blobid(parent)));
            }

            unique_ref<RustFileBlob> RustFsBlobStore::createFileBlob(const BlockId &parent)
            {
                return make_unique_ref<RustFileBlob>(_fsBlobStore->create_file_blob(*cast_blobid(parent)));
            }

            unique_ref<RustSymlinkBlob> RustFsBlobStore::createSymlinkBlob(const boost::filesystem::path &target, const BlockId &parent)
            {
                return make_unique_ref<RustSymlinkBlob>(_fsBlobStore->create_symlink_blob(*cast_blobid(parent), target.string()));
            }

            optional<unique_ref<RustFsBlob>> RustFsBlobStore::load(const BlockId &blockId)
            {
                auto blob = _fsBlobStore->load(*cast_blobid(blockId));
                if (!blob->has_value())
                {
                    return boost::none;
                }
                return make_unique_ref<RustFsBlob>(blob->extract_value());
            }

            uint64_t RustFsBlobStore::numBlocks() const
            {
                return _fsBlobStore->num_blocks();
            }

            uint64_t RustFsBlobStore::estimateSpaceForNumBlocksLeft() const
            {
                return _fsBlobStore->estimate_space_for_num_blocks_left();
            }

            uint64_t RustFsBlobStore::virtualBlocksizeBytes() const
            {
                return _fsBlobStore->virtual_block_size_bytes();
            }

            uint8_t RustFsBlobStore::loadBlockDepth(const blockstore::BlockId &blockId) const {
                return _fsBlobStore->load_block_depth(*cast_blobid(blockId));
            }
        }
    }
}
