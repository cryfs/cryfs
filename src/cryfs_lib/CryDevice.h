#pragma once
#ifndef CRYFS_LIB_CRYDEVICE_H_
#define CRYFS_LIB_CRYDEVICE_H_

#include <blockstore/interface/BlockStore.h>
#include "CryConfig.h"

#include <boost/filesystem.hpp>
#include <fspp/fs_interface/Device.h>

#include "fspp/utils/macros.h"

namespace cryfs {

namespace bf = boost::filesystem;

class CryDevice: public fspp::Device {
public:
  static constexpr size_t DIR_BLOCKSIZE = 4096;

  CryDevice(std::unique_ptr<CryConfig> config, std::unique_ptr<blockstore::BlockStore> blockStore);
  virtual ~CryDevice();

  void statfs(const boost::filesystem::path &path, struct ::statvfs *fsstat) override;

  blockstore::BlockWithKey CreateBlock(size_t size);

private:
  blockstore::Key GetOrCreateRootKey(CryConfig *config);
  blockstore::Key CreateRootBlockAndReturnKey();
  std::unique_ptr<fspp::Node> Load(const bf::path &path) override;

  std::unique_ptr<blockstore::BlockStore> _block_store;

  blockstore::Key _root_key;

  DISALLOW_COPY_AND_ASSIGN(CryDevice);
};

}

#endif
