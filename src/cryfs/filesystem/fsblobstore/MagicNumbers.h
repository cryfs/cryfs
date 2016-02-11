#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_MAGICNUMBERS_H_
#define MESSMER_CRYFS_FILESYSTEM_FSBLOBSTORE_MAGICNUMBERS_H_

namespace cryfs {
namespace fsblobstore {

//TODO enum class
enum MagicNumbers {
  DIR = 0x00,
  FILE = 0x01,
  SYMLINK = 0x02
};

}
}



#endif
