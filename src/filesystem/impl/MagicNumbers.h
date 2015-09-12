#pragma once
#ifndef CRYFS_LIB_IMPL_MAGICNUMBERS_H_
#define CRYFS_LIB_IMPL_MAGICNUMBERS_H_

namespace cryfs {

//TODO enum class
enum MagicNumbers {
  DIR = 0x00,
  FILE = 0x01,
  SYMLINK = 0x02
};

}



#endif
