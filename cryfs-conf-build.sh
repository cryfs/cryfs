#!/bin/bash -ex
#
# This file has two purposes:
#  - simplify/automate configuration and build of CryFS for specific
#    environment (e.g., all the dependent libraries are in /opt/local);
#  - serve as a run-script for "git bisect" to help finding the
#    offending commit.
#

rm -rf build
mkdir -p build
cd build

CMAKEFLAGS="-DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=off"

CMAKEFLAGS="${CMAKEFLAGS} -DCMAKE_INSTALL_PREFIX=/opt/local"

CMAKEFLAGS="${CMAKEFLAGS} -DBoost_USE_STATIC_LIBS=off -DCRYPTOPP_LIB_PATH=/opt/local/lib -DCRYFS_UPDATE_CHECKS=off"

cmake .. ${CMAKEFLAGS} -DCMAKE_C_FLAGS="-I/opt/local/include"
make -j 4
#make check
cd ..
exit 0

