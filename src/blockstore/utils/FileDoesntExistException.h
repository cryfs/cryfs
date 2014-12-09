#pragma once
#ifndef BLOCKSTORE_UTILS_FILEDOESNTEXISTEXCEPTION_H_
#define BLOCKSTORE_UTILS_FILEDOESNTEXISTEXCEPTION_H_

#include <boost/filesystem/path.hpp>

#include <stdexcept>

namespace blockstore {

class FileDoesntExistException: public std::runtime_error {
public:
  FileDoesntExistException(const boost::filesystem::path &filepath);
  virtual ~FileDoesntExistException();
};

}

#endif
