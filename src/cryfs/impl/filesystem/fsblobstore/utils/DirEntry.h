#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRY_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRY_H

#include <blockstore/utils/BlockId.h>
#include <fspp/fs_interface/Dir.h>
#include <fspp/fs_interface/Types.h>
#include <cpp-utils/system/time.h>
#include <sys/stat.h>

namespace cryfs {
    namespace fsblobstore {

        class DirEntry final {
        public:
            DirEntry(fspp::Dir::EntryType type, std::string name, const blockstore::BlockId &blockId) :
            _type(type), _name(std::move(name)), _blockId(blockId) {};

            void serialize(uint8_t* dest) const;
            size_t serializedSize() const;
            static const char *deserializeAndAddToVector(const char *pos, std::vector<DirEntry> *result);

            fspp::Dir::EntryType type() const;

            const std::string &name() const;
            void setName(const std::string &value);

            const blockstore::BlockId &blockId() const;

        private:

            fspp::Dir::EntryType _type;
            std::string _name;
            blockstore::BlockId _blockId;
        };


        inline fspp::Dir::EntryType DirEntry::type() const {
            return _type;
        }

        inline const std::string &DirEntry::name() const {
            return _name;
        }

        inline const blockstore::BlockId &DirEntry::blockId() const {
            return _blockId;
        }


        inline void DirEntry::setName(const std::string &value) {
            _name = value;
        }

        struct DirEntryWithMetaData {

          DirEntryWithMetaData(fspp::Dir::EntryType type, const std::string &name, const blockstore::BlockId &blockId, fspp::mode_t mode,
                               fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                               timespec lastMetadataChangeTime)
            : _type(type), _name(name), _blockId(blockId), _mode(mode), _uid(uid), _gid(gid), _lastAccessTime(lastAccessTime),
                    _lastModificationTime(lastModificationTime), _lastMetadataChangeTime(lastMetadataChangeTime) {
              ASSERT((_mode.hasFileFlag() && _type == fspp::Dir::EntryType::FILE) ||
                     (_mode.hasDirFlag() && _type == fspp::Dir::EntryType::DIR) ||
                     (_mode.hasSymlinkFlag() && _type == fspp::Dir::EntryType::SYMLINK), "Unknown mode in entry");
            }

          fspp::Dir::EntryType _type;
          std::string _name;
          blockstore::BlockId _blockId;
          fspp::mode_t _mode;
          fspp::uid_t _uid;
          fspp::gid_t _gid;
          timespec _lastAccessTime;
          timespec _lastModificationTime;
          timespec _lastMetadataChangeTime;
          static const char *deserializeAndAddToVector(const char *pos, std::vector<DirEntryWithMetaData> *result);
          static timespec _deserializeTimeValue(const char **pos);


        };
    }
}

#endif
