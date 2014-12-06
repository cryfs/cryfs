#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOBSTORE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOBSTORE_H_

#include "blobstore/interface/helpers/BlobStoreWithRandomKeys.h"

#include <boost/filesystem.hpp>

#include "fspp/utils/macros.h"

#include <mutex>

namespace blobstore {
namespace ondisk {

class OnDiskBlobStore: public BlobStoreWithRandomKeys {
public:
  OnDiskBlobStore(const boost::filesystem::path &rootdir);

  bool exists(const std::string &key) override;
  std::unique_ptr<BlobWithKey> create(const std::string &key, size_t size) override;
  std::unique_ptr<Blob> load(const std::string &key) override;

private:
  const boost::filesystem::path _rootdir;

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlobStore);
};

} /* namespace ondisk */
} /* namespace blobstore */

#endif
