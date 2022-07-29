#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_RUSTBRIDGE_HELPERS_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_RUSTBRIDGE_HELPERS_H_

#include "cxxbridge/cryfs-cppbridge/src/blobstore.rs.h"

namespace blobstore
{
    namespace rust
    {
        namespace helpers
        {
            inline ::rust::Box<bridge::BlobId> cast_blobid(const blockstore::BlockId &blobId)
            {
                return bridge::new_blobid(blobId.data().as_array());
            }
            inline blockstore::BlockId cast_blobid(const bridge::BlobId &blobId)
            {
                return blockstore::BlockId::FromBinary(blobId.data().data());
            }
            // inline ::rust::Slice<const uint8_t> cast_data(const cpputils::Data &data)
            // {
            //     return ::rust::Slice<const uint8_t>{static_cast<const uint8_t *>(data.data()), data.size()};
            // }
            inline cpputils::Data cast_data(const ::blobstore::rust::bridge::Data *dataObj)
            {
                auto data = dataObj->data();
                cpputils::Data result(data.size());
                std::memcpy(result.data(), data.data(), data.size());
                return result;
            }
        }
    }
}

#endif
