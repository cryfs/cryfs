#!/bin/bash

set -ev

# Use new clang on linux
if [ "${TRAVIS_OS_NAME}" == "linux" ] && [ "$CXX" == "clang++" ]; then
  echo Switch to Clang 3.7
  export CXX="clang++-3.7" CC="clang-3.7"
else
  echo Do not switch to Clang 3.7 because we are either not Linux or not Clang.
fi

# If using gcc on mac, actually use it ("gcc" just links to clang, but "gcc-4.8" is gcc, https://github.com/travis-ci/travis-ci/issues/2423)
# Note: This must be here and not in install.sh, because environment variables can't be passed between scripts.
if [ "${TRAVIS_OS_NAME}" == "osx" ] && [ "${CXX}" == "g++" ]; then
  echo Switch to actual g++ and not just the AppleClang symlink
  export CXX="g++-7" CC="gcc-7"
else
  echo Do not switch to actual g++ because we are either not osx or not g++
fi

# Setup ccache
export PATH="/usr/local/opt/ccache/libexec:$PATH"
export CCACHE_COMPILERCHECK=content
export CCACHE_COMPRESS=1
export CCACHE_SLOPPINESS=include_file_mtime
ccache --max-size=512M
ccache --show-stats

# Detect number of CPU cores
export NUMCORES=`grep -c ^processor /proc/cpuinfo`
if [ ! -n "$NUMCORES" ]; then
  export NUMCORES=`sysctl -n hw.ncpu`
fi
echo Using $NUMCORES cores

echo Using CXX compiler $CXX and C compiler $CC

# Setup target directory
mkdir cmake
cd cmake
cmake --version

# Build
cmake .. -DBUILD_TESTING=on -DCMAKE_BUILD_TYPE=Debug
make -j$NUMCORES

ccache --show-stats

# Test
./test/gitversion/gitversion-test
./test/cpp-utils/cpp-utils-test
./test/parallelaccessstore/parallelaccessstore-test
./test/blockstore/blockstore-test
./test/blobstore/blobstore-test
./test/cryfs/cryfs-test

# TODO Also run on osx once fixed
if [ "${TRAVIS_OS_NAME}" == "linux" ]; then
  ./test/fspp/fspp-test
  ./test/cryfs-cli/cryfs-cli-test
fi
