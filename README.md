# CryFS [![Build Status](https://travis-ci.org/cryfs/cryfs.svg?branch=master)](https://travis-ci.org/cryfs/cryfs)
CryFS encrypts your files, so you can safely store them anywhere. It works well together with cloud services like Dropbox, iCloud, OneDrive and others.
See [https://www.cryfs.org](https://www.cryfs.org).

Install latest release
======================

Easy install (Ubuntu and Debian)
--------------------------------

    wget -O - https://www.cryfs.org/install.sh | sudo bash

Manual install (Ubuntu)
-----------------------

    # Add apt key
    wget -O - https://www.cryfs.org/apt.key | sudo apt-key add -

    # Add apt repository
    sudo sh -c "echo \"deb http://apt.cryfs.org/ubuntu `lsb_release -s -c` main\" > /etc/apt/sources.list.d/cryfs.list"
    
    # Install cryfs 
    sudo apt-get update
    sudo apt-get install cryfs

Manual install (Debian)
-----------------------
    # Add apt key
    wget -O - https://www.cryfs.org/apt.key | sudo apt-key add -

    # Add apt repository
    sudo sh -c "echo \"deb http://apt.cryfs.org/debian `lsb_release -s -c` main\" > /etc/apt/sources.list.d/cryfs.list"
    
    # Install cryfs 
    sudo apt-get update
    sudo apt-get install cryfs
    
GUI
===
If you want to use a GUI to mount your CryFS volumes, take a look at [zuluCrypt](http://mhogomchungu.github.io/zuluCrypt/). You can simply drag&drop your CryFS encrypted directory into the zuluMount application to mount it.

Building from source
====================

Requirements
------------
  - Git (for getting the source code)
  - GCC version >= 4.8 or Clang >= 3.5
  - CMake version >= 2.8
  - libcurl4 (including development headers) 
  - Boost libraries version >= 1.56 (including development headers)
    - filesystem
    - system
    - chrono
    - program_options
    - thread
  - Crypto++ version >= 5.6.3 (including development headers)
  - SSL development libraries (including development headers, e.g. libssl-dev)
  - libFUSE version >= 2.8.6 (including development headers), on Mac OS X instead install osxfuse from https://osxfuse.github.io/
  - Python >= 2.7

You can use the following commands to install these requirements

        # Ubuntu
        $ sudo apt-get install git g++ cmake libcurl4-openssl-dev libboost-filesystem-dev libboost-system-dev libboost-chrono-dev libboost-program-options-dev libboost-thread-dev libcrypto++-dev libssl-dev libfuse-dev python
        
        # Fedora
        TODO
        
        # Macintosh
        brew install cmake boost cryptopp openssl

Build & Install
---------------
 
 1. Clone repository

        $ git clone https://github.com/cryfs/cryfs.git cryfs
        $ cd cryfs

 2. Build

        $ mkdir cmake && cd cmake
        $ cmake ..
        $ make
        
 3. Install

        $ sudo make install
        
You can pass the following variables to CMake (using *-Dvariablename=value*):
 - -D**CMAKE_BUILD_TYPE**=[Release|Debug]: Whether to run code optimization or add debug symbols. Default: Release
 - -D**BUILD_TESTING**=[on|off]: Whether to build the test cases (can take a long time). Default: off
 
On most systems, CMake should find the libraries automatically.
If this doesn't work for you, you can use the following CMake variables:
 - -D**FUSE_LIB_PATH**=[path]: Path to the directory containing the fuse library (or osxfuse library on Mac)
 - -D**CRYPTOPP_LIB_PATH**=[path]: Path to the directory containing the Crypto++ library

If your build can't find header files (this was reported on Mac OS X for openssl and fuse headers),
you can add include paths using *-DCMAKE_CXX_FLAGS="-I/path/to/header/files -I/path/to/other/header/files"*.


Creating .deb packages
----------------------

There are additional requirements if you want to create .deb packages. They are:
 - CMake version >= 3.3
 - (optional) rpmbuild

 1. Clone repository

        $ git clone https://github.com/cryfs/cryfs.git cryfs
        $ cd cryfs

 2. Build

        $ mkdir cmake && cd cmake
        $ cmake .. -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=off
        $ make package
