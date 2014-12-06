#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOBSTORE_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_ONDISKBLOBSTORE_H_

#include "blobstore/interface/BlobStore.h"

#include <boost/filesystem.hpp>

#include "fspp/utils/macros.h"

#include <mutex>

namespace blobstore {
namespace ondisk {
class OnDiskBlob;

class OnDiskBlobStore: public BlobStore {
public:
  OnDiskBlobStore(const boost::filesystem::path &rootdir);

  BlobWithKey create(size_t size) override;
  std::unique_ptr<Blob> load(const std::string &key) override;

private:
  std::string _generateKey();
  std::string _generateRandomKey();
  const boost::filesystem::path _rootdir;

  std::mutex _generate_key_mutex;

  DISALLOW_COPY_AND_ASSIGN(OnDiskBlobStore);
};

} /* namespace ondisk */
} /* namespace blobstore */

#endif
