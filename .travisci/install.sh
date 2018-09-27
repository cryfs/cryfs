#!/bin/bash

set -e

# Install dependencies
if [ "${TRAVIS_OS_NAME}" == "linux" ]; then
  ./.travisci/install_boost.sh
fi

# Install newer GCC if we're running on GCC osx
if [ "${TRAVIS_OS_NAME}" == "osx" ] && [ "${CXX}" == "g++" ]; then
    # We need to uninstall oclint because it creates a /usr/local/include/c++ symlink that clashes with the gcc5 package
    # see https://github.com/Homebrew/homebrew-core/issues/21172
    brew cask uninstall oclint
    brew install gcc@7
fi

if [ "${TRAVIS_OS_NAME}" == "osx" ]; then
  brew cask install osxfuse
  brew install libomp
fi

# By default, travis only fetches the newest 50 commits. We need more in case we're further from the last version tag, so the build doesn't fail because it can't generate the version number.
git fetch --unshallow

#  Use /dev/urandom when /dev/random is accessed, because travis doesn't have enough entropy
if [ "${TRAVIS_OS_NAME}" == "linux" ]; then
  sudo cp -a /dev/urandom /dev/random
fi

# Setup ccache
brew install ccache
