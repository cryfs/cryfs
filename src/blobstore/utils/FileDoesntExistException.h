#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_FILEDOESNTEXISTEXCEPTION_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_FILEDOESNTEXISTEXCEPTION_H_

#include <boost/filesystem/path.hpp>

#include <stdexcept>

namespace blobstore {

class FileDoesntExistException: public std::runtime_error {
public:
  FileDoesntExistException(const boost::filesystem::path &filepath);
  virtual ~FileDoesntExistException();
};

} /* namespace blobstore */

#endif
