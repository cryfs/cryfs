#pragma once
#ifndef MESSMER_BLOCKSTORE_UTILS_FILEDOESNTEXISTEXCEPTION_H_
#define MESSMER_BLOCKSTORE_UTILS_FILEDOESNTEXISTEXCEPTION_H_

#include <boost/filesystem/path.hpp>

#include <stdexcept>

namespace blockstore {

class FileDoesntExistException final: public std::runtime_error {
public:
  explicit FileDoesntExistException(const boost::filesystem::path &filepath);
  ~FileDoesntExistException() override;
};

}

#endif
