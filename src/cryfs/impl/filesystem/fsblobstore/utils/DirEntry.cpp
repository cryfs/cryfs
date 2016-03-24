#include "DirEntry.h"

using std::vector;
using std::string;
using blockstore::Key;

namespace cryfs {
    namespace fsblobstore {
        void DirEntry::serialize(uint8_t *dest) const {
            ASSERT(
                    ((_type == fspp::Dir::EntryType::FILE) && S_ISREG(_mode) && !S_ISDIR(_mode) && !S_ISLNK(_mode)) ||
                    ((_type == fspp::Dir::EntryType::DIR) && !S_ISREG(_mode) && S_ISDIR(_mode) && !S_ISLNK(_mode)) ||
                    ((_type == fspp::Dir::EntryType::SYMLINK) && !S_ISREG(_mode) && !S_ISDIR(_mode) && S_ISLNK(_mode))
                    , "Wrong mode bit set for this type: " + std::to_string(_mode & S_IFREG) + ", " + std::to_string(
                    _mode & S_IFDIR) + ", " + std::to_string(_mode & S_IFLNK) + ", " + std::to_string(static_cast<uint8_t>(_type))
            );
            unsigned int offset = 0;
            offset += _serializeUint8(dest + offset, static_cast<uint8_t>(_type));
            offset += _serializeUint32(dest + offset, _mode);
            offset += _serializeUint32(dest + offset, _uid);
            offset += _serializeUint32(dest + offset, _gid);
            offset += _serializeTimeValue(dest + offset, _lastAccessTime);
            offset += _serializeTimeValue(dest + offset, _lastModificationTime);
            offset += _serializeTimeValue(dest + offset, _lastMetadataChangeTime);
            offset += _serializeString(dest + offset, _name);
            offset += _serializeKey(dest + offset, _key);
            ASSERT(offset == serializedSize(), "Didn't write correct number of elements");
        }

        const char *DirEntry::deserializeAndAddToVector(const char *pos, vector<DirEntry> *result) {
            fspp::Dir::EntryType type = static_cast<fspp::Dir::EntryType>(_deserializeUint8(&pos));
            mode_t mode = _deserializeUint32(&pos);
            uid_t uid = _deserializeUint32(&pos);
            gid_t gid = _deserializeUint32(&pos);
            timespec lastAccessTime = _deserializeTimeValue(&pos);
            timespec lastModificationTime = _deserializeTimeValue(&pos);
            timespec lastMetadataChangeTime = _deserializeTimeValue(&pos);
            string name = _deserializeString(&pos);
            Key key = _deserializeKey(&pos);

            result->emplace_back(type, name, key, mode, uid, gid, lastAccessTime, lastModificationTime, lastMetadataChangeTime);
            return pos;
        }

        unsigned int DirEntry::_serializeTimeValue(uint8_t *dest, timespec value) {
            unsigned int offset = 0;
            *reinterpret_cast<uint64_t*>(dest+offset) = value.tv_sec;
            offset += sizeof(uint64_t);
            *reinterpret_cast<uint32_t*>(dest+offset) = value.tv_nsec;
            offset += sizeof(uint32_t);
            ASSERT(offset == _serializedTimeValueSize(), "serialized to wrong size");
            return offset;
        }

        size_t DirEntry::_serializedTimeValueSize() {
            return sizeof(uint64_t) + sizeof(uint32_t);
        }

        timespec DirEntry::_deserializeTimeValue(const char **pos) {
            timespec value;
            value.tv_sec = *(uint64_t*)(*pos);
            *pos += sizeof(uint64_t);
            value.tv_nsec = *(uint32_t*)(*pos);
            *pos += sizeof(uint32_t);
            return value;
        }

        unsigned int DirEntry::_serializeUint8(uint8_t *dest, uint8_t value) {
            *reinterpret_cast<uint8_t*>(dest) = value;
            return sizeof(uint8_t);
        }

        uint8_t DirEntry::_deserializeUint8(const char **pos) {
            uint8_t value = *(uint8_t*)(*pos);
            *pos += sizeof(uint8_t);
            return value;
        }

        unsigned int DirEntry::_serializeUint32(uint8_t *dest, uint32_t value) {
            *reinterpret_cast<uint32_t*>(dest) = value;
            return sizeof(uint32_t);
        }

        uint32_t DirEntry::_deserializeUint32(const char **pos) {
            uint32_t value = *(uint32_t*)(*pos);
            *pos += sizeof(uint32_t);
            return value;
        }

        unsigned int DirEntry::_serializeString(uint8_t *dest, const string &value) {
            std::memcpy(dest, value.c_str(), value.size()+1);
            return value.size() + 1;
        }

        string DirEntry::_deserializeString(const char **pos) {
            size_t length = strlen(*pos);
            string value(*pos, length);
            *pos += length + 1;
            return value;
        }

        unsigned int DirEntry::_serializeKey(uint8_t *dest, const Key &key) {
            key.ToBinary(dest);
            return key.BINARY_LENGTH;
        }

        Key DirEntry::_deserializeKey(const char **pos) {
            Key key = Key::FromBinary(*pos);
            *pos += Key::BINARY_LENGTH;
            return key;
        }

        size_t DirEntry::serializedSize() const {
            return 1 + sizeof(uint32_t) + sizeof(uint32_t) + sizeof(uint32_t) + 3*_serializedTimeValueSize() + (
                    _name.size() + 1) + _key.BINARY_LENGTH;
        }
    }
}
