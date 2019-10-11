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
            size_t _serialize(void* dst, const DataType& obj) {
                cpputils::serialize<DataType>(dst, obj);
                return sizeof(DataType);
            }

            template<typename DataType>
            DataType _deserialize(const char** src) {
                auto result = cpputils::deserialize<DataType>(*src);
                *src += sizeof(DataType);
                return result;
            }


            unsigned int _serializeString(uint8_t *dest, const string &value) {
                std::memcpy(dest, value.c_str(), value.size()+1);
                return value.size() + 1;
            }

            string _deserializeString(const char **pos) {
                size_t length = strlen(*pos);
                string value(*pos, length);
                *pos += length + 1;
                return value;
            }

            unsigned int _serializeBlockId(uint8_t *dest, const BlockId &blockId) {
                blockId.ToBinary(dest);
                return blockId.BINARY_LENGTH;
            }

            BlockId _deserializeBlockId(const char **pos) {
                BlockId blockId = BlockId::FromBinary(*pos);
                *pos += BlockId::BINARY_LENGTH;
                return blockId;
            }
        }

        void DirEntry::serialize(uint8_t *dest) const {
            unsigned int offset = 0;
            offset += _serialize<uint8_t>(dest + offset, static_cast<uint8_t>(_type));
            offset += _serializeString(dest + offset, _name);
            offset += _serializeBlockId(dest + offset, _blockId);
            ASSERT(offset == serializedSize(), "Didn't write correct number of elements");
        }

        const char *DirEntry::deserializeAndAddToVector(const char *pos, vector<DirEntry> *result) {
            auto type = static_cast<fspp::Dir::NodeType>(_deserialize<uint8_t>(&pos));
            string name = _deserializeString(&pos);
            BlockId blockId = _deserializeBlockId(&pos);

            result->emplace_back(type, name, blockId);
            return pos;
        }



        size_t DirEntry::serializedSize() const {
            return 1 + (_name.size() + 1) + _blockId.BINARY_LENGTH;
        }

        const char *DirEntryWithMetaData::deserializeAndAddToVector(const char *pos, vector<DirEntryWithMetaData> *result) {
          auto type = static_cast<fspp::Dir::NodeType>(_deserialize<uint8_t>(&pos));
          fspp::mode_t mode = fspp::mode_t(_deserialize<uint32_t>(&pos));
          fspp::uid_t uid = fspp::uid_t(_deserialize<uint32_t>(&pos));
          fspp::gid_t gid = fspp::gid_t(_deserialize<uint32_t>(&pos));
          timespec lastAccessTime = _deserializeTimeValue(&pos);
          timespec lastModificationTime = _deserializeTimeValue(&pos);
          timespec lastMetadataChangeTime = _deserializeTimeValue(&pos);
          string name = _deserializeString(&pos);
          BlockId blockId = _deserializeBlockId(&pos);

          std::cerr << "name: " << name << std::endl;
          result->emplace_back(type, name, blockId, mode, uid, gid, lastAccessTime, lastModificationTime, lastMetadataChangeTime);
          return pos;
        }

        timespec DirEntryWithMetaData::_deserializeTimeValue(const char **pos) {
          timespec value{};
          value.tv_sec = _deserialize<uint64_t>(pos);
          value.tv_nsec = _deserialize<uint32_t>(pos);
          return value;
        }

    }
}
