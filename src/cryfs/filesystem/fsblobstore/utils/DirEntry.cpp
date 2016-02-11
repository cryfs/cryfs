#include "DirEntry.h"

using std::vector;
using blockstore::Key;

namespace cryfs {
    namespace fsblobstore {

        void DirEntry::serialize(uint8_t *dest) const {
            unsigned int offset = 0;
            *(dest+offset) = static_cast<uint8_t>(type);
            offset += 1;

            std::memcpy(dest+offset, name.c_str(), name.size()+1);
            offset += name.size() + 1;

            key.ToBinary(dest+offset);
            offset += key.BINARY_LENGTH;

            *reinterpret_cast<uid_t*>(dest+offset) = uid;
            offset += sizeof(uid_t);

            *reinterpret_cast<gid_t*>(dest+offset) = gid;
            offset += sizeof(gid_t);

            *reinterpret_cast<mode_t*>(dest+offset) = mode;
            offset += sizeof(mode_t);

            //TODO Persist times, see comment in deserializeAndAddToVector()
            //static_assert(sizeof(timespec) == 16, "Ensure platform independence of the serialization");
            //*reinterpret_cast<timespec*>(dest+offset) = lastAccessTime;
            //offset += sizeof(timespec);
            //*reinterpret_cast<timespec*>(dest+offset) = lastModificationTime;
            //offset += sizeof(timespec);
            //*reinterpret_cast<timespec*>(dest+offset) = lastMetadataChangeTime;
            //offset += sizeof(timespec);

            ASSERT(offset == serializedSize(), "Didn't write correct number of elements");
        }

        size_t DirEntry::serializedSize() const {
            //TODO Persist times, see comment in deserializeAndAddToVector()
            //return 1 + (name.size() + 1) + key.BINARY_LENGTH + sizeof(uid_t) + sizeof(gid_t) + sizeof(mode_t) + 3*sizeof(timespec);
            return 1 + (name.size() + 1) + key.BINARY_LENGTH + sizeof(uid_t) + sizeof(gid_t) + sizeof(mode_t);
        }

        const char *DirEntry::deserializeAndAddToVector(const char *pos, vector<DirEntry> *result) {
            // Read type magic number (whether it is a dir or a file)
            fspp::Dir::EntryType type =
                    static_cast<fspp::Dir::EntryType>(*reinterpret_cast<const unsigned char*>(pos));
            pos += 1;

            size_t namelength = strlen(pos);
            std::string name(pos, namelength);
            pos += namelength + 1;

            Key key = Key::FromBinary(pos);
            pos += Key::BINARY_LENGTH;

            uid_t uid = *(uid_t*)pos;
            pos += sizeof(uid_t);

            gid_t gid = *(gid_t*)pos;
            pos += sizeof(gid_t);

            mode_t mode = *(mode_t*)pos;
            pos += sizeof(mode_t);

            //TODO Persist times. This breaks compatibility though - so change cryfs::InnerConfig::HEADER
            //     This is already implemented, but I commented it out for now, because it would break compatibility.
            //timespec lastAccessTime = *(timespec*)pos;
            //pos += sizeof(timespec);
            //timespec lastModificationTime = *(timespec*)pos;
            //pos += sizeof(timespec);
            //timespec lastMetadataChangeTime = *(timespec*)pos;
            //pos += sizeof(timespec);
            timespec lastAccessTime;
            clock_gettime(CLOCK_REALTIME, &lastAccessTime);
            timespec lastModificationTime = lastAccessTime;
            timespec lastMetadataChangeTime = lastAccessTime;

            result->emplace_back(type, name, key, mode, uid, gid, lastAccessTime, lastModificationTime, lastMetadataChangeTime);
            return pos;
        }

    }
}
