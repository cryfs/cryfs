#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRYLIST_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRYLIST_H

#include <cpp-utils/data/Data.h>
#include "DirEntry.h"
#include <vector>
#include <string>
#include "TimestampUpdateBehavior.h"

//TODO Address elements by name instead of by blockId when accessing them. Who knows whether there is two hard links for the same blob.

namespace cryfs {
    namespace fsblobstore {

        class DirEntryList final {
        public:
            using const_iterator = std::vector<DirEntry>::const_iterator;

            enum class AddOver {
              ADD, OVERWRITE
            };

            DirEntryList();

            cpputils::Data serialize() const;
            static cpputils::Data serializeExternal(const std::vector<DirEntry>& entries);
            void deserializeFrom(const void *data, uint64_t size);

            void add(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::NodeType entryType);
            AddOver addOrOverwrite(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::NodeType entryType,
                     const std::function<void (const DirEntry &entry)>& onOverwritten);
            void rename(const blockstore::BlockId &blockId, const std::string &name, const std::function<void (const DirEntry &entry)>& onOverwritten);
            boost::optional<const DirEntry&> get(const std::string &name) const;
            boost::optional<const DirEntry&> get(const blockstore::BlockId &blockId) const;
            void remove(const std::string &name);
            void remove(const blockstore::BlockId &blockId);

            size_t size() const;
            const_iterator begin() const;
            const_iterator end() const;

        private:
            uint64_t _serializedSize() const;
            static uint64_t _serializedSizeExternal(const std::vector<DirEntry>&);
            bool _hasChild(const std::string &name) const;
            std::vector<DirEntry>::iterator _findByName(const std::string &name);
            std::vector<DirEntry>::const_iterator _findByName(const std::string &name) const;
            std::vector<DirEntry>::iterator _findById(const blockstore::BlockId &blockId);
            std::vector<DirEntry>::const_iterator _findById(const blockstore::BlockId &blockId) const;

          std::vector<DirEntry>::iterator _findLowerBound(const blockstore::BlockId &blockId);
            std::vector<DirEntry>::iterator _findFirst(const blockstore::BlockId &hint, const std::function<bool (const DirEntry&)>& pred);
            void _add(const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::NodeType entryType);
            void _overwrite(std::vector<DirEntry>::iterator entry, const std::string &name, const blockstore::BlockId &blobId, fspp::Dir::NodeType entryType);
            static void _checkAllowedOverwrite(fspp::Dir::NodeType oldType, fspp::Dir::NodeType newType);

            std::vector<DirEntry> _entries;

            DISALLOW_COPY_AND_ASSIGN(DirEntryList);
        };

    }
}

#endif
