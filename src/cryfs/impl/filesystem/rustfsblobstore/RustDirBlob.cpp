#include "RustDirBlob.h"
#include <cryfs/impl/filesystem/rustfsblobstore/helpers.h>
#include <fspp/fs_interface/FuseErrnoException.h>

using blockstore::BlockId;
using boost::optional;
using cpputils::make_unique_ref;
using cpputils::unique_ref;
using cryfs::fsblobstore::rust::helpers::cast_blobid;
using cryfs::fsblobstore::rust::helpers::cast_timespec;
using cryfs::fsblobstore::rust::helpers::cast_entry_type;
using cryfs::fsblobstore::rust::helpers::cast_entry;
using std::function;
using std::string;
using std::vector;

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            void _maybe_throw(const ::rust::Box<bridge::FsResult> &result)
            {
                if (result->is_err())
                {
                    if (result->is_errno_error()) {
                        throw fspp::fuse::FuseErrnoException(result->err_errno());
                    } else {
                        throw std::runtime_error(std::string("Error from Rust: ") + result->err_message().c_str());
                    }
                }
            }

            RustDirBlob::RustDirBlob(::rust::Box<bridge::RustDirBlobBridge> dirBlob)
                : _dirBlob(std::move(dirBlob))
            {
            }

            RustDirBlob::~RustDirBlob()
            {
                _dirBlob->async_drop();
            }

            void RustDirBlob::flush()
            {
                _dirBlob->flush();
            }

            BlockId RustDirBlob::blockId() const
            {
                return cast_blobid(*_dirBlob->blob_id());
            }

            BlockId RustDirBlob::parent() const
            {
                return cast_blobid(*_dirBlob->parent());
            }

            size_t RustDirBlob::NumChildren() const {
                return _dirBlob->num_entries();
            }

            void RustDirBlob::AppendChildrenTo(vector<fspp::Dir::Entry> *result) const {
                auto entries = _dirBlob->entries();
                result->reserve(result->size() + entries.size());
                for (const auto &entry : entries) {
                    result->push_back(cast_entry(entry));
                }
            }

            optional<unique_ref<RustDirEntry>> RustDirBlob::GetChild(const string &name) const
            {
                auto entry = _dirBlob->entry_by_name(name);
                if (!entry->has_value())
                {
                    return boost::none;
                }
                return make_unique_ref<RustDirEntry>(entry->extract_value());
            }

            optional<unique_ref<RustDirEntry>> RustDirBlob::GetChild(const BlockId &blockId) const
            {
                auto entry = _dirBlob->entry_by_id(*cast_blobid(blockId));
                if (!entry->has_value())
                {
                    return boost::none;
                }
                return make_unique_ref<RustDirEntry>(entry->extract_value());
            }

            void RustDirBlob::RenameChild(const BlockId &blockId, const string &newName, function<void(const BlockId &blockId)> onOverwritten)
            {
                _maybe_throw(
                    _dirBlob->rename_entry(*cast_blobid(blockId), newName, std::make_unique<CxxCallbackWithBlobId>(
                        [onOverwritten = std::move(onOverwritten)](const fsblobstore::rust::bridge::FsBlobId &blobId) {
                            onOverwritten(cast_blobid(blobId));
                        })
                    )
                );
            }

            void RustDirBlob::maybeUpdateAccessTimestampOfChild(const blockstore::BlockId& blockId, fspp::TimestampUpdateBehavior atimeUpdateBehavior) {
                bridge::AtimeUpdateBehavior behavior;
                switch (*atimeUpdateBehavior) {
                    case fspp::detail::TimestampUpdateBehaviorBase::NOATIME:
                        behavior = bridge::AtimeUpdateBehavior::Noatime;
                        break;
                    case fspp::detail::TimestampUpdateBehaviorBase::STRICTATIME:
                        behavior = bridge::AtimeUpdateBehavior::Strictatime;
                        break;
                    case fspp::detail::TimestampUpdateBehaviorBase::RELATIME:
                        behavior = bridge::AtimeUpdateBehavior::Relatime;
                        break;
                    case fspp::detail::TimestampUpdateBehaviorBase::NODIRATIME_STRICTATIME:
                        behavior = bridge::AtimeUpdateBehavior::NodiratimeStrictatime;
                        break;
                    case fspp::detail::TimestampUpdateBehaviorBase::NODIRATIME_RELATIME:
                        behavior = bridge::AtimeUpdateBehavior::NodiratimeRelatime;
                        break;
                    default:
                        throw std::runtime_error("Unknown atime update behavior");
                }
                _maybe_throw(
                    _dirBlob->maybe_update_access_timestamp_of_entry(*cast_blobid(blockId), behavior)
                );
            }

            void RustDirBlob::updateModificationTimestampOfChild(const blockstore::BlockId &blockId) {
                _maybe_throw(
                    _dirBlob->update_modification_timestamp_of_entry(*cast_blobid(blockId))
                );
            }

            void RustDirBlob::setModeOfChild(const blockstore::BlockId &blockId, fspp::mode_t mode) {
                _maybe_throw(
                    _dirBlob->set_mode_of_entry(*cast_blobid(blockId), mode.value())
                );
            }

            void RustDirBlob::setUidGidOfChild(const blockstore::BlockId &blockId, fspp::uid_t uid, fspp::gid_t gid) {
                const auto option_uid = (uid == fspp::uid_t(-1)) ? bridge::new_none_u32() : bridge::new_some_u32(uid.value());
                const auto option_gid = (gid == fspp::gid_t(-1)) ? bridge::new_none_u32() : bridge::new_some_u32(gid.value());
                
                _maybe_throw(
                    _dirBlob->set_uid_gid_of_entry(*cast_blobid(blockId), *option_uid, *option_gid)
                );
            }

            void RustDirBlob::setAccessTimesOfChild(const blockstore::BlockId &blockId, const timespec &lastAccessTime, const timespec &lastModificationTime) {
                _maybe_throw(
                    _dirBlob->set_access_times_of_entry(*cast_blobid(blockId), cast_timespec(lastAccessTime), cast_timespec(lastModificationTime))
                );
            }

            void RustDirBlob::AddChildDir(const std::string &name, const BlockId &blobId, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
                _maybe_throw(
                    _dirBlob->add_entry_dir(name, *cast_blobid(blobId), mode.value(), uid.value(), gid.value(), cast_timespec(lastAccessTime), cast_timespec(lastModificationTime))
                );
            }

            void RustDirBlob::AddChildFile(const std::string &name, const BlockId &blobId, fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
                _maybe_throw(
                    _dirBlob->add_entry_file(name, *cast_blobid(blobId), mode.value(), uid.value(), gid.value(), cast_timespec(lastAccessTime), cast_timespec(lastModificationTime))
                );
            }

            void RustDirBlob::AddChildSymlink(const std::string &name, const BlockId &blobId, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime) {
                _maybe_throw(
                    _dirBlob->add_entry_symlink(name, *cast_blobid(blobId), uid.value(), gid.value(), cast_timespec(lastAccessTime), cast_timespec(lastModificationTime))
                );
            }

            void RustDirBlob::AddOrOverwriteChild(const std::string &name, const BlockId &blobId, fspp::Dir::EntryType entryType,
                                  fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                                  std::function<void (const blockstore::BlockId &blockId)> onOverwritten) {
                _maybe_throw(
                    _dirBlob->add_or_overwrite_entry(name, *cast_blobid(blobId), cast_entry_type(entryType), mode.value(), uid.value(), gid.value(), cast_timespec(lastAccessTime), cast_timespec(lastModificationTime), std::make_unique<CxxCallbackWithBlobId>(
                        [onOverwritten = std::move(onOverwritten)](const fsblobstore::rust::bridge::FsBlobId &blobId) {
                            onOverwritten(cast_blobid(blobId));
                        })
                    )
                );
            }

            void RustDirBlob::RemoveChild(const std::string &name) {
                _maybe_throw(
                    _dirBlob->remove_entry_by_name(name)
                );
            }

            void RustDirBlob::RemoveChildIfExists(const blockstore::BlockId &blockId) {
                _dirBlob->remove_entry_by_id_if_exists(*cast_blobid(blockId));
            }
        }
    }
}