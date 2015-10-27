#pragma once
#ifndef MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGLOADER_H_
#define MESSMER_CRYFS_SRC_CONFIG_CRYCONFIGLOADER_H_

#include <messmer/cpp-utils/pointer/unique_ref.h>
#include <boost/filesystem.hpp>
#include "CryConfigFile.h"
#include "CryCipher.h"
#include "CryConfigCreator.h"
#include <messmer/cpp-utils/crypto/kdf/Scrypt.h>

namespace cryfs {

class CryConfigLoader {
public:
  CryConfigLoader(cpputils::unique_ref<cpputils::Console> console, cpputils::RandomGenerator &keyGenerator, std::function<std::string()> askPassword);
  CryConfigLoader(CryConfigLoader &&rhs) = default;

  template<class SCryptSettings = cpputils::SCryptDefaultSettings>
  boost::optional<CryConfigFile> loadOrCreate(const boost::filesystem::path &filename);

private:
    boost::optional<CryConfigFile> _loadConfig(const boost::filesystem::path &filename);
    template<class SCryptSettings>
    CryConfigFile _createConfig(const boost::filesystem::path &filename);

    CryConfigCreator _creator;
    std::function<std::string()> _askPassword;

    DISALLOW_COPY_AND_ASSIGN(CryConfigLoader);
};

template<class SCryptSettings>
boost::optional<CryConfigFile> CryConfigLoader::loadOrCreate(const boost::filesystem::path &filename) {
    if (boost::filesystem::exists(filename)) {
        return _loadConfig(filename);
    } else {
        return _createConfig<SCryptSettings>(filename);
    }
}

template<class SCryptSettings>
CryConfigFile CryConfigLoader::_createConfig(const boost::filesystem::path &filename) {
    auto config = _creator.create();
    //TODO Ask confirmation if using insecure password (<8 characters)
    std::string password = _askPassword();
    return CryConfigFile::create<SCryptSettings>(filename, std::move(config), password);
}

}

#endif
