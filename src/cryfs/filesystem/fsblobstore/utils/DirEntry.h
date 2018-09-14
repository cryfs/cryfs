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
            DirEntry(fspp::Dir::EntryType type, const std::string &name, const blockstore::BlockId &blockId, fspp::mode_t mode,
                  fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                  timespec lastMetadataChangeTime);

            void serialize(uint8_t* dest) const;
            size_t serializedSize() const;
            static const char *deserializeAndAddToVector(const char *pos, std::vector<DirEntry> *result);

            fspp::Dir::EntryType type() const;
            void setType(fspp::Dir::EntryType value);

            const std::string &name() const;
            void setName(const std::string &value);

            const blockstore::BlockId &blockId() const;

            fspp::mode_t mode() const;
            void setMode(fspp::mode_t value);

            fspp::uid_t uid() const;
            void setUid(fspp::uid_t value);

            fspp::gid_t gid() const;
            void setGid(fspp::gid_t value);

            timespec lastAccessTime() const;
            void setLastAccessTime(timespec value);

            timespec lastModificationTime() const;
            void setLastModificationTime(timespec value);

            timespec lastMetadataChangeTime() const;

        private:

            void _updateLastMetadataChangeTime();

            fspp::Dir::EntryType _type;
            std::string _name;
            blockstore::BlockId _blockId;
            fspp::mode_t _mode;
            fspp::uid_t _uid;
            fspp::gid_t _gid;
            timespec _lastAccessTime;
            timespec _lastModificationTime;
            timespec _lastMetadataChangeTime;
        };

        inline DirEntry::DirEntry(fspp::Dir::EntryType type, const std::string &name, const blockstore::BlockId &blockId, fspp::mode_t mode,
            fspp::uid_t uid, fspp::gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
            timespec lastMetadataChangeTime)
                : _type(type), _name(name), _blockId(blockId), _mode(mode), _uid(uid), _gid(gid), _lastAccessTime(lastAccessTime),
                _lastModificationTime(lastModificationTime), _lastMetadataChangeTime(lastMetadataChangeTime) {
            switch (_type) {
                case fspp::Dir::EntryType::FILE:
                    _mode.addFileFlag();
                    break;
                case fspp::Dir::EntryType::DIR:
                    _mode.addDirFlag();
                    break;
                case fspp::Dir::EntryType::SYMLINK:
                    _mode.addSymlinkFlag();
                    break;
            }
            ASSERT((_mode.hasFileFlag() && _type == fspp::Dir::EntryType::FILE) ||
                   (_mode.hasDirFlag() && _type == fspp::Dir::EntryType::DIR) ||
                   (_mode.hasSymlinkFlag() && _type == fspp::Dir::EntryType::SYMLINK), "Unknown mode in entry");
        }

        inline fspp::Dir::EntryType DirEntry::type() const {
            return _type;
        }

        inline const std::string &DirEntry::name() const {
            return _name;
        }

        inline const blockstore::BlockId &DirEntry::blockId() const {
            return _blockId;
        }

        inline fspp::mode_t DirEntry::mode() const {
            return _mode;
        }

        inline fspp::uid_t DirEntry::uid() const {
            return _uid;
        }

        inline fspp::gid_t DirEntry::gid() const {
            return _gid;
        }

        inline timespec DirEntry::lastAccessTime() const {
            return _lastAccessTime;
        }

        inline timespec DirEntry::lastModificationTime() const {
            return _lastModificationTime;
        }

        inline timespec DirEntry::lastMetadataChangeTime() const {
            return _lastMetadataChangeTime;
        }

        inline void DirEntry::setType(fspp::Dir::EntryType value) {
            _type = value;
            _updateLastMetadataChangeTime();
        }

        inline void DirEntry::setName(const std::string &value) {
            _name = value;
            _updateLastMetadataChangeTime();
        }

        inline void DirEntry::setMode(fspp::mode_t value) {
            _mode = value;
            _updateLastMetadataChangeTime();
        }

        inline void DirEntry::setUid(fspp::uid_t value) {
            _uid = value;
            _updateLastMetadataChangeTime();
        }

        inline void DirEntry::setGid(fspp::gid_t value) {
            _gid = value;
            _updateLastMetadataChangeTime();
        }

        inline void DirEntry::setLastAccessTime(timespec value) {
            _lastAccessTime = value;
        }

        inline void DirEntry::setLastModificationTime(timespec value) {
            _lastModificationTime = value;
            _updateLastMetadataChangeTime();
        }

        inline void DirEntry::_updateLastMetadataChangeTime() {
            _lastMetadataChangeTime = cpputils::time::now();
        }

    }
}

#endif
