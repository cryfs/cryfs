#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOBSTORE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOBSTORE_H_

#include "blobstore/interface/BlobStore.h"

#include <boost/filesystem/path.hpp>

namespace blobstore {
namespace ondisk {
class OnDiskBlob;

class OnDiskBlobStore: public BlobStore {
public:
  OnDiskBlobStore(const boost::filesystem::path &rootdir);

  std::unique_ptr<Blob> create(const std::string &key, size_t size) override;
  std::unique_ptr<Blob> load(const std::string &key) override;

private:
  const boost::filesystem::path _rootdir;
};

} /* namespace ondisk */
} /* namespace blobstore */

#endif
