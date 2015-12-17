# cryfs [![Build Status](https://travis-ci.org/cryfs/cryfs.svg?branch=master)](https://travis-ci.org/cryfs/cryfs)
Cryptographic filesystem for the cloud

See http://www.cryfs.org

This repository contains the filesystem implementation. There are submodules in the following repositores:

  - [Blockstore](https://github.com/cryfs/blockstore): Store (encrypted) fixed-size blocks of data in different backends
  - [Blobstore](https://github.com/cryfs/blobstore): Store resizeable blobs of data using blocks from a blockstore
  - [ParallelAccessStore](https://github.com/cryfs/parallelaccessstore): Concurrency primitive for Blockstore/Blobstore
  - [Fs++](https://github.com/cryfs/fspp): Implement a file system against a platform independent interface

Install latest release
======================

Easy install (Ubuntu and Debian)
--------------------------------

    wget -O - http://www.cryfs.org/install.sh | sudo bash

Manual install (Ubuntu)
-----------------------

    # Add apt key
    wget -O - http://www.cryfs.org/apt.key | sudo apt-key add -

    # Add apt repository
    sudo sh -c "echo \"deb http://apt.cryfs.org/ubuntu `lsb_release -s -c` main\" > /etc/apt/sources.list.d/cryfs.list"
    
    # Install cryfs 
    sudo apt-get update
    sudo apt-get install cryfs

Manual install (Debian)
-----------------------
    # Add apt key
    wget -O - http://www.cryfs.org/apt.key | sudo apt-key add -

    # Add apt repository
    sudo sh -c "echo \"deb http://apt.cryfs.org/debian `lsb_release -s -c` main\" > /etc/apt/sources.list.d/cryfs.list"
    
    # Install cryfs 
    sudo apt-get update
    sudo apt-get install cryfs
    

Building from source
====================

Requirements
------------
  - [biicode](https://www.biicode.com/downloads)

        # After installing, call
        $ bii setup:cpp

  - GCC version >= 4.8 or Clang (TODO which minimal version?)
  - libFUSE >= 2.8.6 (including development headers)

        # Ubuntu
        $ sudo apt-get install libfuse-dev
        
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

        $ bii init -L
        $ bii configure -D CMAKE_BUILD_TYPE=Release
        $ bii build
        
 3. (if build failed) Biicode can have a bug sometimes where the first call to configure fails. If that happens, just call it again.

 4. Install

        $ cd bii/build/messmer_cryfs
        $ sudo make install

You can pass normal make parameters after a double dash.
This can for example be used to add "-j5" to compile with 5 build threads in parallel:

        $ bii build -- -j5
