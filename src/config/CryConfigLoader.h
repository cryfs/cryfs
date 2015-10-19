#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGLOADER_H_
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGLOADER_H_

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <boost/filesystem/path.hpp>
#include "CryConfigFile.h"
#include "CryCipher.h"
#include "CryConfigCreator.h"

namespace cryfs {

class CryConfigLoader {
public:
  CryConfigLoader();
  explicit CryConfigLoader(cpputils::unique_ref<cpputils::Console> console);

  CryConfigFile loadOrCreate(const boost::filesystem::path &filename);
  CryConfigFile createNew(const boost::filesystem::path &filename);

  //This methods are only for testing purposes, because creating weak keys is much faster than creating strong keys.
  CryConfigFile loadOrCreateForTest(const boost::filesystem::path &filename);
  CryConfigFile createNewForTest(const boost::filesystem::path &filename);

private:
  CryConfigCreator _creator;
};

}

#endif
