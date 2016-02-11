# cryfs [![Build Status](https://travis-ci.org/cryfs/cryfs.svg?branch=master)](https://travis-ci.org/cryfs/cryfs)
CryFS encrypts your files, so you can safely store them anywhere. It works well together with cloud services like Dropbox, iCloud, OneDrive and others.
See [https://www.cryfs.org](https://www.cryfs.org).

This repository contains the filesystem implementation. There are submodules in the following repositores:

  - [Blockstore](https://github.com/cryfs/blockstore): Store (encrypted) fixed-size blocks of data in different backends
  - [Blobstore](https://github.com/cryfs/blobstore): Store resizeable blobs of data using blocks from a blockstore
  - [ParallelAccessStore](https://github.com/cryfs/parallelaccessstore): Concurrency primitive for Blockstore/Blobstore
  - [Fs++](https://github.com/cryfs/fspp): Implement a file system against a platform independent interface

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
    

Building from source
====================

Requirements
------------
  - GCC version >= 4.8 or Clang (TODO which minimal version?)
  - CMake version >= 3.3
  - libcurl4 (including development headers) 
  - Boost libraries filesystem, system, chrono, program_options, thread in version >= 1.56
  - Crypto++ >= 5.6.3 (TODO Lower minimal version possible?)
  - libFUSE >= 2.8.6 (including development headers)

        # Ubuntu
        $ sudo apt-get install libfuse-dev
        TODO Other install commands
        
        # Fedora
        TODO
        
        # Macintosh
        TODO

Build
-----
 
 1. Clone repository

        $ git clone git@github.com:cryfs/cryfs.git cryfs
        $ cd cryfs

 2. Build

        $ mkdir cmake && cd cmake
        $ cmake .. -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=off
        $ make
        
 3. Install

        $ cd bii/build/messmer_cryfs
        $ sudo make install
