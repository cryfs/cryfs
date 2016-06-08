#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRYLIST_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRYLIST_H

#include <cpp-utils/data/Data.h>
#include "DirEntry.h"
#include <vector>
#include <string>

//TODO Address elements by name instead of by key when accessing them. Who knows whether there is two hard links for the same blob.

namespace cryfs {
    namespace fsblobstore {

        class DirEntryList final {
        public:
            using const_iterator = std::vector<DirEntry>::const_iterator;

            DirEntryList();

            cpputils::Data serialize() const;
            void deserializeFrom(const void *data, uint64_t size);

            void add(const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType entryType,
                     mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime);
            void addOrOverwrite(const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType entryType,
                     mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                     std::function<void (const blockstore::Key &key)> onOverwritten);
            void rename(const blockstore::Key &key, const std::string &name, std::function<void (const blockstore::Key &key)> onOverwritten);
            boost::optional<const DirEntry&> get(const std::string &name) const;
            boost::optional<const DirEntry&> get(const blockstore::Key &key) const;
            void remove(const std::string &name);
            void remove(const blockstore::Key &key);

            size_t size() const;
            const_iterator begin() const;
            const_iterator end() const;

            void setMode(const blockstore::Key &key, mode_t mode);
            bool setUidGid(const blockstore::Key &key, uid_t uid, gid_t gid);
            void setAccessTimes(const blockstore::Key &key, timespec lastAccessTime, timespec lastModificationTime);
            void updateAccessTimestampForChild(const blockstore::Key &key);
            void updateModificationTimestampForChild(const blockstore::Key &key);

        private:
            uint64_t _serializedSize() const;
            bool _hasChild(const std::string &name) const;
            std::vector<DirEntry>::iterator _findByName(const std::string &name);
            std::vector<DirEntry>::const_iterator _findByName(const std::string &name) const;
            std::vector<DirEntry>::iterator _findByKey(const blockstore::Key &key);
            std::vector<DirEntry>::const_iterator _findByKey(const blockstore::Key &key) const;
            std::vector<DirEntry>::iterator _findUpperBound(const blockstore::Key &key);
            std::vector<DirEntry>::iterator _findLowerBound(const blockstore::Key &key);
            std::vector<DirEntry>::iterator _findFirst(const blockstore::Key &hint, std::function<bool (const DirEntry&)> pred);
            void _add(const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType entryType,
                     mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime);
            void _overwrite(std::vector<DirEntry>::iterator entry, const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType entryType,
                      mode_t mode, uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime);
            static void _checkAllowedOverwrite(fspp::Dir::EntryType oldType, fspp::Dir::EntryType newType);

            std::vector<DirEntry> _entries;

            DISALLOW_COPY_AND_ASSIGN(DirEntryList);
        };

    }
}

#endif
