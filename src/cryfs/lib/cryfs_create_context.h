#pragma once
#ifndef CRYFS_CRYFS_CREATE_CONTEXT_H
#define CRYFS_CRYFS_CREATE_CONTEXT_H

#include "../cryfs.h"
#include <string>
#include <boost/optional.hpp>
#include <boost/filesystem/path.hpp>
#include "../impl/config/CryConfigFile.h"
#include "../impl/filesystem/CryDevice.h"

struct cryfs_create_context final {
public:
  cryfs_create_context(cryfs_api_context *api_context);

  cryfs_status set_basedir(const std::string &basedir);
  cryfs_status set_cipher(const std::string &cipher);
  cryfs_status set_password(const std::string &password);
  cryfs_status set_externalconfig(const std::string &configfile);
  cryfs_status create(cryfs_mount_handle **handle);

  cryfs_status free();

private:
  cryfs_api_context *_api_context;

  boost::optional<boost::filesystem::path> _basedir;
  boost::optional<std::string> _cipher;
  boost::optional<std::string> _password;
  boost::optional<boost::filesystem::path> _configfile;

  DISALLOW_COPY_AND_ASSIGN(cryfs_create_context);
};

#endif
