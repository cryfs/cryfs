#include <cpp-utils/data/SerializationHelper.h>
#include "DirEntry.h"
#include <cstdint>

using std::vector;
using std::string;
using blockstore::BlockId;

namespace cryfs {
    namespace fsblobstore {

        namespace {
            template<typename DataType>
            size_t serialize_(void* dst, const DataType& obj) {
                cpputils::serialize<DataType>(dst, obj);
                return sizeof(DataType);
            }

            template<typename DataType>
            DataType deserialize_(const char** src) {
                DataType result = cpputils::deserialize<DataType>(*src);
                *src += sizeof(DataType);
                return result;
            }

            constexpr size_t serializedTimeValueSize_() {
                return sizeof(uint64_t) + sizeof(uint32_t);
            }

            unsigned int serializeTimeValue_(uint8_t *dest, timespec value) {
                unsigned int offset = 0;
                offset += serialize_<uint64_t>(dest + offset, value.tv_sec);
                offset += serialize_<uint32_t>(dest + offset, value.tv_nsec);
                ASSERT(offset == serializedTimeValueSize_(), "serialized to wrong size");
                return offset;
            }

            timespec deserializeTimeValue_(const char **pos) {
                timespec value{};
                value.tv_sec = deserialize_<uint64_t>(pos);
                value.tv_nsec = deserialize_<uint32_t>(pos);
                return value;
            }

            unsigned int serializeString_(uint8_t *dest, const string &value) {
                std::memcpy(dest, value.c_str(), value.size()+1);
                return value.size() + 1;
            }

            string deserializeString_(const char **pos) {
                size_t length = strlen(*pos);
                string value(*pos, length);
                *pos += length + 1;
                return value;
            }

            unsigned int serializeBlockId_(uint8_t *dest, const BlockId &blockId) {
                blockId.ToBinary(dest);
                return blockId.BINARY_LENGTH;
            }

            BlockId deserializeBlockId_(const char **pos) {
                BlockId blockId = BlockId::FromBinary(*pos);
                *pos += BlockId::BINARY_LENGTH;
                return blockId;
            }
        }

        void DirEntry::serialize(uint8_t *dest) const {
            ASSERT(
                    ((_type == fspp::Dir::EntryType::FILE) && _mode.hasFileFlag() && !_mode.hasDirFlag() && !_mode.hasSymlinkFlag()) ||
                    ((_type == fspp::Dir::EntryType::DIR) && !_mode.hasFileFlag() && _mode.hasDirFlag() && !_mode.hasSymlinkFlag()) ||
                    ((_type == fspp::Dir::EntryType::SYMLINK) && !_mode.hasFileFlag() && !_mode.hasDirFlag() && _mode.hasSymlinkFlag())
                    , "Wrong mode bit set for this type: " + std::to_string(_mode.hasFileFlag()) + ", " + std::to_string(
                    _mode.hasDirFlag()) + ", " + std::to_string(_mode.hasSymlinkFlag()) + ", " + std::to_string(static_cast<uint8_t>(_type))
            );
            unsigned int offset = 0;
            offset += serialize_<uint8_t>(dest + offset, static_cast<uint8_t>(_type));
            offset += serialize_<uint32_t>(dest + offset, _mode.value());
            offset += serialize_<uint32_t>(dest + offset, _uid.value());
            offset += serialize_<uint32_t>(dest + offset, _gid.value());
            offset += serializeTimeValue_(dest + offset, _lastAccessTime);
            offset += serializeTimeValue_(dest + offset, _lastModificationTime);
            offset += serializeTimeValue_(dest + offset, _lastMetadataChangeTime);
            offset += serializeString_(dest + offset, _name);
            offset += serializeBlockId_(dest + offset, _blockId);
            ASSERT(offset == serializedSize(), "Didn't write correct number of elements");
        }

        const char *DirEntry::deserializeAndAddToVector(const char *pos, vector<DirEntry> *result) {
            fspp::Dir::EntryType type = static_cast<fspp::Dir::EntryType>(deserialize_<uint8_t>(&pos));
            fspp::mode_t mode = fspp::mode_t(deserialize_<uint32_t>(&pos));
            fspp::uid_t uid = fspp::uid_t(deserialize_<uint32_t>(&pos));
            fspp::gid_t gid = fspp::gid_t(deserialize_<uint32_t>(&pos));
            timespec lastAccessTime = deserializeTimeValue_(&pos);
            timespec lastModificationTime = deserializeTimeValue_(&pos);
            timespec lastMetadataChangeTime = deserializeTimeValue_(&pos);
            string name = deserializeString_(&pos);
            BlockId blockId = deserializeBlockId_(&pos);

            result->emplace_back(type, name, blockId, mode, uid, gid, lastAccessTime, lastModificationTime, lastMetadataChangeTime);
            return pos;
        }

        size_t DirEntry::serializedSize() const {
            return 1 + sizeof(uint32_t) + sizeof(uint32_t) + sizeof(uint32_t) + 3*serializedTimeValueSize_() + (
                    _name.size() + 1) + _blockId.BINARY_LENGTH;
        }
    }
}
