# CryFS [![Build Status](https://travis-ci.org/cryfs/cryfs.svg?branch=master)](https://travis-ci.org/cryfs/cryfs) [![CircleCI](https://circleci.com/gh/cryfs/cryfs/tree/master.svg?style=svg)](https://circleci.com/gh/cryfs/cryfs/tree/master) [![Build status](https://ci.appveyor.com/api/projects/status/84ouutflsnap9dlv/branch/master?svg=true)](https://ci.appveyor.com/project/smessmer/cryfs/branch/master)

CryFS encrypts your files, so you can safely store them anywhere. It works well together with cloud services like Dropbox, iCloud, OneDrive and others.
See [https://www.cryfs.org](https://www.cryfs.org).

Install latest release
======================

Linux
------

This only works for Ubuntu 17.04 and later, and Debian Stretch and later.
You can also use CryFS on older versions of these distributions by following the **Building from source** instructions below.

    sudo apt install cryfs

OSX
----

CryFS is distributed via Homebrew and MacPorts.

If you use Homebrew:

    brew cask install osxfuse
    brew install cryfs

If you use MacPorts (not available for OSX 10.15 at the moment):

    port install cryfs

    
Windows (experimental)
----------------------

CryFS has experimental Windows support since the 0.10 release series. To install it, do:

1. Install [DokanY](https://github.com/dokan-dev/dokany/releases)
2. Install [Microsoft Visual C++ Redistributable for Visual Studio 2019](https://support.microsoft.com/en-us/help/2977003/the-latest-supported-visual-c-downloads)
3. Install [CryFS](https://www.cryfs.org/#download)

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
  - GCC version >= 6.5 or Clang >= 4.0
  - CMake version >= 3.6
  - Conan package manager
  - libcurl4 (including development headers)
  - SSL development libraries (including development headers, e.g. libssl-dev)
  - libFUSE version >= 2.8.6 (including development headers), on Mac OS X instead install osxfuse from https://osxfuse.github.io/
  - Python >= 3.5
  - OpenMP

You can use the following commands to install these requirements

        # Ubuntu
        $ sudo apt install git g++ cmake make libcurl4-openssl-dev libssl-dev libfuse-dev python python3-pip
        $ sudo pip3 install conan

        # Fedora
        $ sudo dnf install git gcc-c++ cmake make libcurl-devel openssl-devel fuse-devel python python3-pip
        $ sudo pip3 install conan

        # Macintosh
        $ brew install cmake openssl libomp
        $ sudo pip3 install conan

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
----------------------------------

Build with Visual Studio 2019 and pass in the following flags to CMake:

    -DDOKAN_PATH=[dokan library location, e.g. "C:\Program Files\Dokan\DokanLibrary-1.2.1"]

If you set these variables correctly in the `CMakeSettings.json` file, you should be able to open the cryfs source folder with Visual Studio 2019.

Troubleshooting
---------------

On most systems, CMake should find the libraries automatically. However, that doesn't always work.

1. **Fuse/Osxfuse library not found**

    Pass in the library path with

        cmake .. -DFUSE_LIB_PATH=/path/to/fuse/or/osxfuse

2. **Fuse/Osxfuse headers not found**

    Pass in the include path with

        cmake .. -DCMAKE_CXX_FLAGS="-I/path/to/fuse/or/osxfuse/headers"

3. **Openssl headers not found**

    Pass in the include path with

        cmake .. -DCMAKE_C_FLAGS="-I/path/to/openssl/include"

4. **OpenMP not found (osx)**

    Either build it without OpenMP

        cmake .. -DDISABLE_OPENMP=on

    but that will cause slower file system mount times (performance after mounting will be unaffected).
    If you installed OpenMP with homebrew or macports, it should be autodetected.
    If that doesn't work for some reason (or you want to use a different installation than the autodetected one),
    pass in these flags:

        cmake .. -DOpenMP_CXX_FLAGS='-Xpreprocessor -fopenmp -I/path/to/openmp/include' -DOpenMP_CXX_LIB_NAMES=omp -DOpenMP_omp_LIBRARY=/path/to/libomp.dylib


Using local dependencies
-------------------------------
Starting with CryFS 0.11, Conan is used for dependency management.
When you build CryFS, Conan downloads the exact version of each dependency library that was also used for development.
All dependencies are linked statically, so there should be no incompatibility with locally installed libraries.
This is the recommended way because it has the highest probability of working correctly.

However, some distributions prefer software packages to be built against dependencies dynamically and against locally installed versions of libraries.
So if you're building a package for such a distribution, you have the option of doing that, at the cost of potential incompatibilities.
If you follow this workflow, please make sure to extensively test your build of CryFS.
You're using a setup that wasn't tested by the CryFS developers.

To use local dependencies, you need to tell the CryFS build how to get these dependencies.
You can do this by writing a small CMake configuration file and passing it to the CryFS build using `-DDEPENDENCY_CONFIG=filename`.
This configuration file needs to define a cmake target for each of the dependencies.

Here's an [example config file](cmake-utils/DependenciesFromConan.cmake) that gets the dependencies from conan.
And here's another [example config file](cmake-utils/DependenciesFromLocalSystem.cmake) that works for getting dependencies that are locally installed in Ubuntu.
You can create your own configuration file to tell the build how to get its dependencies and, for example, mix and match. Get some dependencies from Conan and others from the local system.


Creating .deb and .rpm packages
-------------------------------

It is recommended to install CryFS using packages, because that allows for an easy way to uninstall it again once you don't need it anymore.

If you want to create a .rpm package, you need to install rpmbuild.

 1. Clone repository

        $ git clone https://github.com/cryfs/cryfs.git cryfs
        $ cd cryfs

 2. Build

        $ mkdir cmake && cd cmake
        $ cmake .. -DCMAKE_BUILD_TYPE=RelWithDebInfo -DBUILD_TESTING=off
        $ make package


Disclaimer
----------------------

In the event of a password leak, you are strongly advised to create a new filesystem and copy all the data over from the previous one. Done this, all copies of the compromised filesystem and config file must be removed (e.g, from the "previous versions" feature of your cloud system) to prevent access to the key (and, as a result, your data) using the leaked password.
