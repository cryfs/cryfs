# cryfs
Cryptographic filesystem for the cloud

See http://www.cryfs.org

This repository contains the filesystem implementation. There are submodules in the following repositores:

  - [Blockstore](https://github.com/cryfs/blockstore): Store (encrypted) fixed-size blocks of data in different backends
  - [Blobstore](https://github.com/cryfs/blobstore): Store resizeable blobs of data using blocks from a blockstore
  - [ParallelAccessStore](https://github.com/cryfs/parallelaccessstore): Concurrency primitive for Blockstore/Blobstore
  - [Fs++](https://github.com/cryfs/fspp): Implement a file system against a platform independent interface

