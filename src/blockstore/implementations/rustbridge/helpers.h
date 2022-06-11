#include "cxxbridge/cryfs-blockstore/src/blockstore/cppbridge.rs.h"

namespace blockstore
{
    namespace rust
    {
        namespace helpers
        {
            inline ::rust::Box<bridge::BlockId> cast_blockid(const BlockId &blockId)
            {
                return bridge::new_blockid(blockId.data().as_array());
            }
            inline BlockId cast_blockid(const bridge::BlockId &blockId)
            {
                return BlockId::FromBinary(blockId.data().data());
            }
            inline ::rust::Slice<const uint8_t> cast_data(const cpputils::Data &data)
            {
                return ::rust::Slice<const uint8_t>{static_cast<const uint8_t *>(data.data()), data.size()};
            }
            inline boost::optional<cpputils::Data> cast_optional_data(const ::blockstore::rust::bridge::OptionData *optionData)
            {
                if (optionData->has_value())
                {
                    auto data = optionData->value();
                    cpputils::Data result(data.size());
                    std::memcpy(result.data(), data.data(), data.size());
                    return result;
                }
                else
                {
                    return boost::none;
                }
            }
        }
    }
}
