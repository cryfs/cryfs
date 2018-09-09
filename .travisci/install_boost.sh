#!/bin/bash

set -e

wget -O boost.tar.bz2 https://sourceforge.net/projects/boost/files/boost/1.56.0/boost_1_56_0.tar.bz2/download
tar -xf boost.tar.bz2
cd boost_1_56_0
# TODO We should use clang as toolchain for building boost when clang is used for building our code
./bootstrap.sh --with-libraries=filesystem,thread,chrono,program_options
sudo ./b2 -d0 -j$NUMCORES install
cd ..
sudo rm -rf boost.tar.bz2 boost_1_56_0
