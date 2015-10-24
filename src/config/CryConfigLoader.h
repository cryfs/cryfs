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
  CryConfigLoader(cpputils::unique_ref<cpputils::Console> console, cpputils::RandomGenerator &keyGenerator, std::function<std::string()> askPassword);
  CryConfigLoader(CryConfigLoader &&rhs) = default;

  CryConfigFile loadOrCreate(const boost::filesystem::path &filename);

private:
    CryConfigFile _loadConfig(const boost::filesystem::path &filename);
    CryConfigFile _createConfig(const boost::filesystem::path &filename);

    CryConfigCreator _creator;
    std::function<std::string()> _askPassword;

    DISALLOW_COPY_AND_ASSIGN(CryConfigLoader);
};

}

#endif
