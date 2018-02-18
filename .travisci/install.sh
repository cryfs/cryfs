#!/bin/bash

set -e

# Use new clang on linux
if [ "${TRAVIS_OS_NAME}" == "linux" ] && [ "$CXX" = "clang++" ]; then
  export CXX="clang++-3.7" CC="clang-3.7"
fi

# If using gcc on mac, actually use it ("gcc" just links to clang, but "gcc-4.8" is gcc, https://github.com/travis-ci/travis-ci/issues/2423)
if [ "${TRAVIS_OS_NAME}" == "osx" ] && ["${CXX}" = "g++" ]; then
  export CXX="g++-4.8" CC="gcc-4.8"
fi

# Install dependencies
if [ "${TRAVIS_OS_NAME}" == "linux" ]; then
  ./.travisci/install_boost.sh
fi

if [ "${TRAVIS_OS_NAME}" == "osx" ]; then
  brew cask install osxfuse
  brew install cryptopp
fi

# By default, travis only fetches the newest 50 commits. We need more in case we're further from the last version tag, so the build doesn't fail because it can't generate the version number.
git fetch --unshallow

#  Use /dev/urandom when /dev/random is accessed, because travis doesn't have enough entropy
if [ "${TRAVIS_OS_NAME}" == "linux" ]; then
  sudo cp -a /dev/urandom /dev/random
fi
