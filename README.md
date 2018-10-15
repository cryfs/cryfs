# CryFS [![Build Status](https://travis-ci.org/cryfs/cryfs.svg?branch=master)](https://travis-ci.org/cryfs/cryfs) [![CircleCI](https://circleci.com/gh/cryfs/cryfs/tree/master.svg?style=svg)](https://circleci.com/gh/cryfs/cryfs/tree/master) [![Build status](https://ci.appveyor.com/api/projects/status/84ouutflsnap9dlv/branch/develop?svg=true)](https://ci.appveyor.com/project/smessmer/cryfs/branch/develop)

CryFS encrypts your files, so you can safely store them anywhere. It works well together with cloud services like Dropbox, iCloud, OneDrive and others.
See [https://www.cryfs.org](https://www.cryfs.org).

Install latest release
======================

This only works for Ubuntu 17.04 and later, and Debian Stretch and later.
You can also use CryFS on older versions of these distributions by following the **Building from source** instructions below.

    sudo apt install cryfs

GUI
===
Theres some GUI applications with CryFS support. You usually have to install the GUI **and** also CryFS itself for it to work.
- [SiriKali](https://mhogomchungu.github.io/sirikali/)
- [Plasma Vault](https://www.kde.org/announcements/plasma-5.11.0.php) in KDE Plasma >= 5.11

Building from source
====================

Requirements
------------
  - Git (for getting the source code)
  - GCC version >= 5.0 or Clang >= 4.0
  - CMake version >= 3.0 
  - libcurl4 (including development headers)
  - Boost libraries version >= 1.56 (including development headers)
    - filesystem
    - system
    - chrono
    - program_options
    - thread
  - SSL development libraries (including development headers, e.g. libssl-dev)
  - libFUSE version >= 2.8.6 (including development headers), on Mac OS X instead install osxfuse from https://osxfuse.github.io/
  - Python >= 2.7
  - OpenMP

You can use the following commands to install these requirements

        # Ubuntu
        $ sudo apt-get install git g++ cmake make libcurl4-openssl-dev libboost-filesystem-dev libboost-system-dev libboost-chrono-dev libboost-program-options-dev libboost-thread-dev libssl-dev libfuse-dev python

        # Fedora
        sudo dnf install git gcc-c++ cmake make libcurl-devel boost-devel boost-static openssl-devel fuse-devel python

        # Macintosh
        brew install cmake boost openssl libomp

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

You can pass the following variables to the *cmake* command (using *-Dvariablename=value*):
 - **-DCMAKE_BUILD_TYPE**=[Release|Debug]: Whether to run code optimization or add debug symbols. Default: Release
 - **-DBUILD_TESTING**=[on|off]: Whether to build the test cases (can take a long time). Default: off
 - **-DCRYFS_UPDATE_CHECKS**=off: Build a CryFS that doesn't check online for updates and security vulnerabilities.

Building on Windows (experimental)
---------------

Build with Visual Studio 2017 and pass in the following flags to CMake:

  -DDOKAN_PATH=[dokan library location, e.g. "C:\Program Files\Dokan\DokanLibrary-1.1.0"]
  -DBOOST_ROOT=[path to root of boost installation]

Troubleshooting
---------------

On most systems, CMake should find the libraries automatically. However, that doesn't always work.

1. **Boost headers not found**

    Pass in the boost include path with

        cmake .. -DBoost_INCLUDE_DIRS=/path/to/boost/headers

    If you want to link boost dynamically (e.g. you don't have the static libraries), use the following:

        cmake .. -DBoost_USE_STATIC_LIBS=off

2. **Fuse/Osxfuse library not found**

    Pass in the library path with

        cmake .. -DFUSE_LIB_PATH=/path/to/fuse/or/osxfuse

3. **Fuse/Osxfuse headers not found**

    Pass in the include path with

        cmake .. -DCMAKE_CXX_FLAGS="-I/path/to/fuse/or/osxfuse/headers"

4. **Openssl headers not found**

    Pass in the include path with

        cmake .. -DCMAKE_C_FLAGS="-I/path/to/openssl/include"

5. **OpenMP not found (osx)**

    Either build it without OpenMP

        cmake .. -DDISABLE_OPENMP=on

    but that will cause slower file system mount times (performance after mounting will be unaffected).
    If you installed OpenMP with homebrew or macports, it should be autodetected.
    If that doesn't work for some reason (or you want to use a different installation than the autodetected one),
    pass in these flags:

        cmake .. -DOpenMP_CXX_FLAGS='-Xpreprocessor -fopenmp -I/path/to/openmp/include' -DOpenMP_CXX_LIB_NAMES=omp -DOpenMP_omp_LIBRARY=/path/to/libomp.dylib


Creating .deb and .rpm packages
-------------------------------

There are additional requirements if you want to create packages. They are:
 - CMake version >= 3.3
 - rpmbuild for creating .rpm package

 1. Clone repository

        $ git clone https://github.com/cryfs/cryfs.git cryfs
        $ cd cryfs

 2. Build

        $ mkdir cmake && cd cmake
        $ cmake .. -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=off
        $ make package


Disclaimer
----------------------

On the event of a password leak, you are strongly advised to create a new filesystem and copy all the data over from the previous one. Done this, all copies of the compromised filesystem and config file must be removed (e.g, from the "previous versions" feature of your cloud system) to prevent access to the key (and, as a result, your data) using the leaked password.
