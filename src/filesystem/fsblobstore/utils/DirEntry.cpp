#include "DirEntry.h"

using std::vector;
using blockstore::Key;

namespace cryfs {
    namespace fsblobstore {

        void DirEntry::serialize(uint8_t *dest) const {
            ASSERT(
                    ((type == fspp::Dir::EntryType::FILE)    &&  S_ISREG(mode) && !S_ISDIR(mode) && !S_ISLNK(mode)) ||
                    ((type == fspp::Dir::EntryType::DIR)     && !S_ISREG(mode) &&  S_ISDIR(mode) && !S_ISLNK(mode)) ||
                    ((type == fspp::Dir::EntryType::SYMLINK) && !S_ISREG(mode) && !S_ISDIR(mode) &&  S_ISLNK(mode))
                    , "Wrong mode bit set for this type: "+std::to_string(mode & S_IFREG)+", "+std::to_string(mode&S_IFDIR)+", "+std::to_string(mode&S_IFLNK)+", "+std::to_string(static_cast<uint8_t>(type))
            );
            unsigned int offset = 0;
            *(dest+offset) = static_cast<uint8_t>(type);
            offset += 1;

            *reinterpret_cast<uint32_t*>(dest+offset) = mode;
            offset += sizeof(uint32_t);

            *reinterpret_cast<uint32_t*>(dest+offset) = uid;
            offset += sizeof(uint32_t);

            *reinterpret_cast<uint32_t*>(dest+offset) = gid;
            offset += sizeof(uint32_t);

            offset += _serializeTimeValue(dest + offset, lastAccessTime);
            offset += _serializeTimeValue(dest + offset, lastModificationTime);
            offset += _serializeTimeValue(dest + offset, lastMetadataChangeTime);

            std::memcpy(dest+offset, name.c_str(), name.size()+1);
            offset += name.size() + 1;

            key.ToBinary(dest+offset);
            offset += key.BINARY_LENGTH;

            ASSERT(offset == serializedSize(), "Didn't write correct number of elements");
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

        size_t DirEntry::serializedSize() const {
            return 1 + sizeof(uint32_t) + sizeof(uint32_t) + sizeof(uint32_t) + 3*_serializedTimeValueSize() + (name.size() + 1) + key.BINARY_LENGTH;
        }

        const char *DirEntry::deserializeAndAddToVector(const char *pos, vector<DirEntry> *result) {
            // Read type magic number (whether it is a dir, file or symlink)
            fspp::Dir::EntryType type = static_cast<fspp::Dir::EntryType>(*reinterpret_cast<const unsigned char*>(pos));
            pos += 1;

            mode_t mode = *(uint32_t*)pos;
            pos += sizeof(uint32_t);

            uid_t uid = *(uint32_t*)pos;
            pos += sizeof(uint32_t);

            gid_t gid = *(uint32_t*)pos;
            pos += sizeof(uint32_t);

            timespec lastAccessTime = _deserializeTimeValue(&pos);
            timespec lastModificationTime = _deserializeTimeValue(&pos);
            timespec lastMetadataChangeTime = _deserializeTimeValue(&pos);

            size_t namelength = strlen(pos);
            std::string name(pos, namelength);
            pos += namelength + 1;

            Key key = Key::FromBinary(pos);
            pos += Key::BINARY_LENGTH;

            result->emplace_back(type, name, key, mode, uid, gid, lastAccessTime, lastModificationTime, lastMetadataChangeTime);
            return pos;
        }
    }
}
