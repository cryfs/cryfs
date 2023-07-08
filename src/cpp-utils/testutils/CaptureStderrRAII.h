#pragma once
#ifndef MESSMER_CPPUTILS_CAPTURESTDERRRAII_H
#define MESSMER_CPPUTILS_CAPTURESTDERRRAII_H

#include <cpp-utils/macros.h>
#include <iostream>
#include <gmock/gmock.h>
#include <regex>

namespace cpputils {

class CaptureStderrRAII final {
public:
  CaptureStderrRAII() : _oldBuffer(std::cerr.rdbuf()) {
    

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
    // TODO For some reason this doesn't work on MSVC
    // EXPECT_THAT(get_stderr(), testing::MatchesRegex(".*" + regex + ".*"));
    EXPECT_TRUE(std::regex_search(get_stderr(), std::regex(regex, std::regex::basic)));
  }

private:
  std::stringstream _buffer;
  std::streambuf *_oldBuffer;

  DISALLOW_COPY_AND_ASSIGN(CaptureStderrRAII);
};

}

#endif
