#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_CXXCALLBACKWITHBLOBID_H_
#define MESSMER_CRYFS_FILESYSTEM_RUSTFSBLOBSTORE_CXXCALLBACKWITHBLOBID_H_

#include <functional>

namespace rust
{
    inline namespace cxxbridge1
    {
        template <class T>
        class Box;
    }
}
namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            namespace bridge
            {
                class FsBlobId;
            }
        }
    }
}

namespace cryfs
{
    namespace fsblobstore
    {
        namespace rust
        {
            class CxxCallbackWithBlobId
            {
            public:
                explicit CxxCallbackWithBlobId(std::function<void(const fsblobstore::rust::bridge::FsBlobId &)> callback)
                    : _callback(std::move(callback)) {}

                void call(const fsblobstore::rust::bridge::FsBlobId &blockid) const
                {
                    _callback(blockid);
                }

            private:
                std::function<void(const fsblobstore::rust::bridge::FsBlobId &)> _callback;
            };
        }
    }
}

#endif
