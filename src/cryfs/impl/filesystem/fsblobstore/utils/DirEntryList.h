#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRYLIST_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRYLIST_H

#include <cpp-utils/data/Data.h>
#include <fspp/fs_interface/Context.h>
#include "DirEntry.h"
#include <vector>
#include <string>

//TODO Address elements by name instead of by blockId when accessing them. Who knows whether there is two hard links for the same blob.

namespace cryfs {
    namespace fsblobstore {

        class DirEntryList final {
        public:
            using const_iterator = std::vector<DirEntry>::const_iterator;

            DirEntryList();

            cpputils::Data serialize() const;
            void deserializeFrom(const void *data, uint64_t size);

            void add(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::EntryType entryType,
                     fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime);
            void addOrOverwrite(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::EntryType entryType,
                     fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                     std::function<void (const blockstore::BlockId &blockId)> onOverwritten);
            void rename(const blockstore::BlockId &blockId, const std::string &name, std::function<void (const blockstore::BlockId &blockId)> onOverwritten);
            boost::optional<const DirEntry&> get(const std::string &name) const;
            boost::optional<const DirEntry&> get(const blockstore::BlockId &blockId) const;
            void remove(const std::string &name);
            void remove(const blockstore::BlockId &blockId);

            size_t size() const;
            const_iterator begin() const;
            const_iterator end() const;

            void setMode(const blockstore::BlockId &blockId, fspp::mode_t mode);
            bool setUidGid(const blockstore::BlockId &blockId, fspp::uid_t uid, fspp::gid_t gid);
            void setAccessTimes(const blockstore::BlockId &blockId, timespec lastAccessTime, timespec lastModificationTime);
            bool updateAccessTimestampForChild(const blockstore::BlockId &blockId, fspp::TimestampUpdateBehavior timestampUpdateBehavior);
            void updateModificationTimestampForChild(const blockstore::BlockId &blockId);

        private:
            uint64_t _serializedSize() const;
            bool _hasChild(const std::string &name) const;
            std::vector<DirEntry>::iterator _findByName(const std::string &name);
            std::vector<DirEntry>::const_iterator _findByName(const std::string &name) const;
            std::vector<DirEntry>::iterator _findById(const blockstore::BlockId &blockId);
            std::vector<DirEntry>::const_iterator _findById(const blockstore::BlockId &blockId) const;
            std::vector<DirEntry>::iterator _findUpperBound(const blockstore::BlockId &blockId);
            std::vector<DirEntry>::iterator _findLowerBound(const blockstore::BlockId &blockId);
            std::vector<DirEntry>::iterator _findFirst(const blockstore::BlockId &hint, std::function<bool (const DirEntry&)> pred);
            void _add(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::EntryType entryType,
                     fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime);
            void _overwrite(std::vector<DirEntry>::iterator entry, const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::EntryType entryType,
                      fspp::mode_t mode, fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime);
            static void _checkAllowedOverwrite(fspp::Dir::EntryType oldType, fspp::Dir::EntryType newType);

            std::vector<DirEntry> _entries;

            DISALLOW_COPY_AND_ASSIGN(DirEntryList);
        };

    }
}

#endif
