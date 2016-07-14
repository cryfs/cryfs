#include "FsBlobRef.h"
#include "CachingFsBlobStore.h"

namespace cryfs {
namespace cachingfsblobstore {

FsBlobRef::~FsBlobRef() {
    if (_baseBlob.isValid()) {
        _fsBlobStore->releaseForCache(std::move(_baseBlob));
    }
}

}
}