#include "RustFsBlob.h"
#include <cryfs/impl/filesystem/rustfsblobstore/helpers.h>

using blockstore::BlockId;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cryfs::fsblobstore::rust::helpers::cast_blobid;

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            RustFsBlob::RustFsBlob(::rust::Box<bridge::RustFsBlobBridge> fsBlob)
                : _fsBlob(std::move(fsBlob))
            {
            }

            RustFsBlob::~RustFsBlob()
            {
                if (_fsBlob != boost::none) {
                    (*_fsBlob)->async_drop();
                }
            }

            fspp::num_bytes_t RustFsBlob::lstat_size() {
                ASSERT(_fsBlob != boost::none, "RustFsBlob::lstat_size() called on moved-from object");
                return fspp::num_bytes_t((*_fsBlob)->lstat_size());
            }

            bool RustFsBlob::isFile() const
            {
                ASSERT(_fsBlob != boost::none, "RustFsBlob::isFile() called on moved-from object");
                return (*_fsBlob)->is_file();
            }

            bool RustFsBlob::isDir() const
            {
                ASSERT(_fsBlob != boost::none, "RustFsBlob::isDir() called on moved-from object");
                return (*_fsBlob)->is_dir();
            }

            bool RustFsBlob::isSymlink() const
            {
                ASSERT(_fsBlob != boost::none, "RustFsBlob::isSymlink() called on moved-from object");
                return (*_fsBlob)->is_symlink();
            }

            unique_ref<RustFileBlob> RustFsBlob::asFile() &&
            {
                ASSERT(_fsBlob != boost::none, "RustFsBlob::asFile() called on moved-from object");
                auto result = make_unique_ref<RustFileBlob>(std::move((*_fsBlob)->to_file()));
                _fsBlob = boost::none;
                return result;
            }

            unique_ref<RustDirBlob> RustFsBlob::asDir() &&
            {
                ASSERT(_fsBlob != boost::none, "RustFsBlob::asDir() called on moved-from object");
                auto result = make_unique_ref<RustDirBlob>(std::move((*_fsBlob)->to_dir()));
                _fsBlob = boost::none;
                return result;
            }

            unique_ref<RustSymlinkBlob> RustFsBlob::asSymlink() &&
            {
                ASSERT(_fsBlob != boost::none, "RustFsBlob::asSymlink() called on moved-from object");
                auto result = make_unique_ref<RustSymlinkBlob>(std::move((*_fsBlob)->to_symlink()));
                _fsBlob = boost::none;
                return result;
            }

            BlockId RustFsBlob::parent() const
            {
                ASSERT(_fsBlob != boost::none, "RustFsBlob::parent() called on moved-from object");
                return cast_blobid(*(*_fsBlob)->parent());
            }

            void RustFsBlob::setParent(const blockstore::BlockId &parent) {
                ASSERT(_fsBlob != boost::none, "RustFsBlob::setParent() called on moved-from object");
                (*_fsBlob)->set_parent(*cast_blobid(parent));
            }

            BlockId RustFsBlob::blockId() const
            {
                ASSERT(_fsBlob != boost::none, "RustFsBlob::blockId() called on moved-from object");
                return cast_blobid(*(*_fsBlob)->blob_id());
            }

            void RustFsBlob::remove() && {
                ASSERT(_fsBlob != boost::none, "RustFsBlob::remove() called on moved-from object");
                (*_fsBlob)->remove();
                _fsBlob = boost::none;
            }

            std::vector<BlockId> RustFsBlob::allBlocks() const {
                auto block_ids = (*_fsBlob)->all_blocks();
                std::vector<BlockId> result;
                result.reserve(block_ids.size());
                for (const auto &block_id : block_ids) {
                    result.emplace_back(cast_blobid(block_id));
                }
                return result;
            }
        }
    }
}