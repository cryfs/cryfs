#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_ONDISK_FILEALREADYEXISTSEXCEPTION_H_
#define BLOCKSTORE_IMPLEMENTATIONS_ONDISK_FILEALREADYEXISTSEXCEPTION_H_

#include <boost/filesystem/path.hpp>

#include <stdexcept>

namespace blockstore {
namespace ondisk {

class FileAlreadyExistsException: public std::runtime_error {
public:
  FileAlreadyExistsException(const boost::filesystem::path &filepath);
  virtual ~FileAlreadyExistsException();
};

}
}

#endif
