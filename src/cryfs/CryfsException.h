#pragma once
#ifndef MESSMER_CRYFS_CRYFSEXCEPTION_H
#define MESSMER_CRYFS_CRYFSEXCEPTION_H

#include "ErrorCodes.h"
#include <stdexcept>
#include <string>

namespace cryfs {

class CryfsException final : public std::runtime_error {
public:
  CryfsException(std::string message, ErrorCode errorCode)
      : std::runtime_error(std::move(message)), _errorCode(errorCode) {}

  ErrorCode errorCode() const {
    return _errorCode;
  }

private:
  ErrorCode _errorCode;
};

}

#endif
