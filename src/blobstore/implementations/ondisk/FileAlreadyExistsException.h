#pragma once
#ifndef BLOBSTORE_IMPLEMENTATIONS_ONDISK_FILEALREADYEXISTSEXCEPTION_H_
#define BLOBSTORE_IMPLEMENTATIONS_ONDISK_FILEALREADYEXISTSEXCEPTION_H_

#include <boost/filesystem/path.hpp>

#include <stdexcept>

namespace blobstore {
namespace ondisk {

class FileAlreadyExistsException: public std::runtime_error {
public:
  FileAlreadyExistsException(const boost::filesystem::path &filepath);
  virtual ~FileAlreadyExistsException();
};

} /* namespace ondisk */
} /* namespace blobstore */

#endif
