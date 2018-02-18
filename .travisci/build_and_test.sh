#!/bin/bash

set -e

# Detect number of CPU cores
export NUMCORES=`grep -c ^processor /proc/cpuinfo`
if [ ! -n "$NUMCORES" ]; then
  export NUMCORES=`sysctl -n hw.ncpu`
fi
echo Using $NUMCORES cores

# Setup target directory
mkdir cmake
cd cmake
cmake --version

# Build
cmake .. -DBUILD_TESTING=on -DCMAKE_BUILD_TYPE=Debug
make -j$NUMCORES

# Test
./test/gitversion/gitversion-test
./test/cpp-utils/cpp-utils-test
./test/parallelaccessstore/parallelaccessstore-test
./test/blockstore/blockstore-test
./test/blobstore/blobstore-test

# TODO Also run on osx once fixed
if [ "${TRAVIS_OS_NAME}" == "linux" ]; then
  ./test/fspp/fspp-test
  ./test/cryfs/cryfs-test
  ./test/cryfs-cli/cryfs-cli-test
fi
