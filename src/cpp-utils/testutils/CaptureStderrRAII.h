#pragma once
#ifndef MESSMER_CPPUTILS_CAPTURESTDERRRAII_H
#define MESSMER_CPPUTILS_CAPTURESTDERRRAII_H

#include <cpp-utils/macros.h>
#include <iostream>
#include <gmock/gmock.h>

namespace cpputils {

class CaptureStderrRAII final {
public:
  CaptureStderrRAII() {
    _oldBuffer = std::cerr.rdbuf();

    // Capture stderr to _buffer
    std::cerr.rdbuf(_buffer.rdbuf());
  }

  ~CaptureStderrRAII() {
    // reset
    std::cerr.rdbuf(_oldBuffer);
  }

  std::string get_stderr() const {
    return _buffer.str();
  }

  void EXPECT_MATCHES(const std::string &regex) {
    EXPECT_THAT(get_stderr(), testing::MatchesRegex(".*" + regex + ".*"));
  }

private:
  std::stringstream _buffer;
  std::streambuf *_oldBuffer;

  DISALLOW_COPY_AND_ASSIGN(CaptureStderrRAII);
};

}

#endif
