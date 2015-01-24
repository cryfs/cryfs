#include "CryDevice.h"

#include "CryDir.h"
#include "CryFile.h"

#include "fspp/fuse/FuseErrnoException.h"
#include "impl/DirBlock.h"

using std::unique_ptr;

using std::unique_ptr;
using std::make_unique;
using std::string;

//TODO Get rid of this in favor of exception hierarchy
using fspp::fuse::CHECK_RETVAL;
using fspp::fuse::FuseErrnoException;

using blockstore::BlockStore;
using blockstore::Key;

namespace cryfs {

CryDevice::CryDevice(unique_ptr<CryConfig> config, unique_ptr<BlockStore> blockStore)
: _block_store(std::move(blockStore)), _root_key(GetOrCreateRootKey(config.get())) {
}

Key CryDevice::GetOrCreateRootKey(CryConfig *config) {
  string root_key = config->RootBlock();
  if (root_key == "") {
    auto key = CreateRootBlockAndReturnKey();
    config->SetRootBlock(key.ToString());
    return key;
  }

  return Key::FromString(root_key);
}

Key CryDevice::CreateRootBlockAndReturnKey() {
  auto rootBlock = _block_store->create(DIR_BLOCKSIZE);
  DirBlock rootDir(std::move(rootBlock));
  rootDir.InitializeEmptyDir();
  return rootBlock->key();
}

CryDevice::~CryDevice() {
}

unique_ptr<fspp::Node> CryDevice::Load(const bf::path &path) {
  printf("Loading %s\n", path.c_str());
  assert(path.is_absolute());

  auto current_block = _block_store->load(_root_key);

  for (const bf::path &component : path.relative_path()) {
    if (!DirBlock::IsDir(*current_block)) {
      throw FuseErrnoException(ENOTDIR);
    }
    unique_ptr<DirBlock> currentDir = make_unique<DirBlock>(std::move(current_block));

    Key childKey = currentDir->GetBlockKeyForName(component.c_str());
    current_block = _block_store->load(childKey);
  }
  if (DirBlock::IsDir(*current_block)) {
    return make_unique<CryDir>(this, std::move(make_unique<DirBlock>(std::move(current_block))));
  } else if (FileBlock::IsFile(*current_block)) {
    return make_unique<CryFile>(std::move(make_unique<FileBlock>(std::move(current_block))));
  } else {
    throw FuseErrnoException(EIO);
  }
}

void CryDevice::statfs(const bf::path &path, struct statvfs *fsstat) {
  throw FuseErrnoException(ENOTSUP);
}

unique_ptr<blockstore::Block> CryDevice::CreateBlock(size_t size) {
  return _block_store->create(size);
}

}
