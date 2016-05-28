#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRY_H
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRY_H

#include <blockstore/utils/Key.h>
#include <fspp/fs_interface/Dir.h>
#include <cpp-utils/system/time.h>

// TODO Implement (and test) atime, noatime, strictatime, relatime mount options

namespace cryfs {
    namespace fsblobstore {

        class DirEntry final {
        public:
            DirEntry(fspp::Dir::EntryType type, const std::string &name, const blockstore::Key &key, mode_t mode,
                  uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
                  timespec lastMetadataChangeTime);

            void serialize(uint8_t* dest) const;
            size_t serializedSize() const;
            static const char *deserializeAndAddToVector(const char *pos, std::vector<DirEntry> *result);

            fspp::Dir::EntryType type() const;
            void setType(fspp::Dir::EntryType value);

            const std::string &name() const;
            void setName(const std::string &value);

            const blockstore::Key &key() const;

            mode_t mode() const;
            void setMode(mode_t value);

            uid_t uid() const;
            void setUid(uid_t value);

            gid_t gid() const;
            void setGid(gid_t value);

            timespec lastAccessTime() const;
            void setLastAccessTime(timespec value);

            timespec lastModificationTime() const;
            void setLastModificationTime(timespec value);

            timespec lastMetadataChangeTime() const;

        private:
            static size_t _serializedTimeValueSize();
            static unsigned int _serializeTimeValue(uint8_t *dest, timespec value);
            static unsigned int _serializeUint8(uint8_t *dest, uint8_t value);
            static unsigned int _serializeUint32(uint8_t *dest, uint32_t value);
            static unsigned int _serializeString(uint8_t *dest, const std::string &value);
            static unsigned int _serializeKey(uint8_t *dest, const blockstore::Key &value);
            static timespec _deserializeTimeValue(const char **pos);
            static uint8_t _deserializeUint8(const char **pos);
            static uint32_t _deserializeUint32(const char **pos);
            static std::string _deserializeString(const char **pos);
            static blockstore::Key _deserializeKey(const char **pos);

            void _updateLastMetadataChangeTime();

            fspp::Dir::EntryType _type;
            std::string _name;
            blockstore::Key _key;
            mode_t _mode;
            uid_t _uid;
            gid_t _gid;
            timespec _lastAccessTime;
            timespec _lastModificationTime;
            timespec _lastMetadataChangeTime;
        };

        inline DirEntry::DirEntry(fspp::Dir::EntryType type, const std::string &name, const blockstore::Key &key, mode_t mode,
            uid_t uid, gid_t gid, timespec lastAccessTime, timespec lastModificationTime,
            timespec lastMetadataChangeTime)
                : _type(type), _name(name), _key(key), _mode(mode), _uid(uid), _gid(gid), _lastAccessTime(lastAccessTime),
                _lastModificationTime(lastModificationTime), _lastMetadataChangeTime(lastMetadataChangeTime) {
            switch (_type) {
                case fspp::Dir::EntryType::FILE:
                    _mode |= S_IFREG;
                    break;
                case fspp::Dir::EntryType::DIR:
                    _mode |= S_IFDIR;
                    break;
                case fspp::Dir::EntryType::SYMLINK:
                    _mode |= S_IFLNK;
                    break;
            }
            ASSERT((S_ISREG(_mode) && _type == fspp::Dir::EntryType::FILE) ||
                   (S_ISDIR(_mode) && _type == fspp::Dir::EntryType::DIR) ||
                   (S_ISLNK(_mode) && _type == fspp::Dir::EntryType::SYMLINK), "Unknown mode in entry");
        }

        inline fspp::Dir::EntryType DirEntry::type() const {
            return _type;
        }

        inline const std::string &DirEntry::name() const {
            return _name;
        }

        inline const blockstore::Key &DirEntry::key() const {
            return _key;
        }

        inline mode_t DirEntry::mode() const {
            return _mode;
        }

        inline uid_t DirEntry::uid() const {
            return _uid;
        }

        inline gid_t DirEntry::gid() const {
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

        inline void DirEntry::setMode(mode_t value) {
            _mode = value;
            _updateLastMetadataChangeTime();
        }

        inline void DirEntry::setUid(uid_t value) {
            _uid = value;
            _updateLastMetadataChangeTime();
        }

        inline void DirEntry::setGid(gid_t value) {
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
