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
  - GCC version >= 4.8 or Clang (TODO which minimal version?)
  - CMake version >= 2.8
  - libcurl4 (including development headers) 
  - Boost libraries version >= 1.56 (including development headers)
    - filesystem
    - system
    - chrono
    - program_options
    - thread
  - Crypto++ version >= 5.6.3 (including development headers) (TODO Lower minimal version possible?)
  - SSL development libraries (including development headers, e.g. libssl-dev)
  - libFUSE version >= 2.8.6 (including development headers)
  - Python >= 2.7

You can use the following commands to install these requirements

        # Ubuntu
        $ sudo apt-get install libcurl4-openssl-dev libboost-filesystem-dev libboost-system-dev libboost-chrono-dev libboost-program-options-dev libboost-thread-dev libcrypto++-dev libssl-dev libfuse-dev python
        
        # Fedora
        TODO
        
        # Macintosh
        TODO

Build & Install
---------------
 
 1. Clone repository

        $ git clone https://github.com/cryfs/cryfs.git cryfs
        $ cd cryfs

 2. Build

        $ mkdir cmake && cd cmake
        $ cmake .. -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=off
        $ make
        
 3. Install

        $ sudo make install


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
