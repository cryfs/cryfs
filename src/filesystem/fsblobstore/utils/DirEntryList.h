#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRYLIST_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRYLIST_H

#include <messmer/cpp-utils/data/Data.h>
#include "DirEntry.h"
#include <vector>
#include <string>

namespace cryfs {
    namespace fsblobstore {

        class DirEntryList {
        public:
            using const_iterator = std::vector<DirEntry>::const_iterator;

            DirEntryList();

            cpputils::Data serialize() const;
            void deserializeFrom(const void *data, uint64_t size);

            void add(const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType entryType,
                     mode_t mode, uid_t uid, gid_t gid);
            const DirEntry &get(const std::string &name) const;
            const DirEntry &get(const blockstore::Key &key) const;
            void remove(const blockstore::Key &key);

            size_t size() const;
            const_iterator begin() const;
            const_iterator end() const;

            void setMode(const blockstore::Key &key, mode_t mode);
            bool setUidGid(const blockstore::Key &key, uid_t uid, gid_t gid);

        private:
            uint64_t _serializedSize() const;
            bool _hasChild(const std::string &name) const;
            std::vector<DirEntry>::iterator _find(const blockstore::Key &key);
            std::vector<DirEntry>::const_iterator _find(const blockstore::Key &key) const;
            std::vector<DirEntry>::iterator _findUpperBound(const blockstore::Key &key);
            std::vector<DirEntry>::iterator _findLowerBound(const blockstore::Key &key);
            std::vector<DirEntry>::iterator _findFirst(const blockstore::Key &hint, std::function<bool (const DirEntry&)> pred);

            std::vector<DirEntry> _entries;

            DISALLOW_COPY_AND_ASSIGN(DirEntryList);
        };

    }
}

#endif
