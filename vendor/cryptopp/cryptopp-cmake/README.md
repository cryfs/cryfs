# Crypto++ CMake

<div align="center">

-+- Build Status -+-

[![Build status - master][build-status-master-badge]][build-matrix]

-+-

[![Latest release][release-badge]][latest-release]
[![Commits][last-commit-badge]][commits]
[![Linux][linux-badge]][latest-release]
[![Windows][windows-badge]][latest-release]
[![Mac OS][macos-badge]][latest-release]
[![License][license-badge]][license]

</div>

## Introduction

This repository contains CMake files for Wei Dai's Crypto++
(<https://github.com/weidai11/cryptopp>) for those who use the library in
[**Modern CMake**](https://cliutils.gitlab.io/modern-cmake/) projects.

See this
[announcement](https://groups.google.com/g/cryptopp-users/c/9oDbTm8qWps) from
crypto++ maintainers.

The emphasis on _Modern_ here is very important. In 2022, we have some really
good solutions to many of the build problems developers used to waste a lot of
time on. CMake in particular has made so much progress to become one of the most
widely used build systems for C++. But, we're not talking about CMake 2.x here.
We're talking about CMake 3.11+ and maybe even CMake 3.24+.

For more details on this topic, see down below...

## Table of Contents

- [Crypto++ CMake](#crypto-cmake)
  - [Introduction](#introduction)
  - [Table of Contents](#table-of-contents)
  - [Before you ask](#before-you-ask)
  - [Versioning principles](#versioning-principles)
  - [Standard usage](#standard-usage)
  - [Using a local copy of crypto++](#using-a-local-copy-of-crypto)
  - [Requesting the master branch of cryptopp](#requesting-the-master-branch-of-cryptopp)
  - [Other ways](#other-ways)
  - [Why Modern CMake?](#why-modern-cmake)

## Before you ask

- **Can you support an older version of CMake? You really don't need 3.21...**

  No.

  This is an opinionated fork with the main purpose being to stay always on a
  recent version of CMake. We believe that the build system should be the latest
  unlike compilers and Operating Systems. If you want to stay on old versions,
  please take a look at the old repo.

- **Can you fix the shared library build? I really like DLLs...**

  Me too, but No.

  Crypto++ does not properly export symbols and manage visibility. You can
  request this feature from the crypto++ project maintainers. The old DLL build
  was only for FIPS version, with limited symbol exports. That version is going
  end-of-life and there is no point from supporting it here.

  If you love DLLs, you can make a [_Wrapper
  DLL_](https://cryptopp.com/wiki/Wrapper_DLL) as explained on crypto++ wiki.

  The CMakeLists.txt in this project are already built for shared and static
  builds, but the shared build is locked until crypto++ is ready for it.

- **Why did you change XXX? It used to work like YYY before...**

  I don't know.

  I use crypto++ in my project, and I use it in a way that I learnt and
  improved over time through experience, extensive reading of other peoples'
  experiences, and sticking as close as possible to modern cmake practices. I'm
  open to new ways and suggestions, especially if they come via a tracked issue,
  a rationale and a pull request. If you have a valid use case, please document
  it in an issue and let's find someone to help make it happen for you. It's
  Open Source :smiley:

## Versioning principles

This project releases track the [crypto++](https://github.com/weidai11/cryptopp)
releases. In other words, every time a new release of _crypto++_ happens, this
project gets updated to take into account changes in source files, compiler
options etc, and will see a new release with the same number than _crypto++_.

At times, bug fixes in this project will happen before a new _crypto++_ release
is published. When a certain number of fixes have been added, and depending on
the criticality of the defects, an additional release tag may be made. These
_patch tags_ will never introduce any additional changes in `crypto++` itself.

Main release tags will have the format: `CRYPTOPP_M_m_p`, while _patch tags_
will have the format `CRYPTOPP_M_m_p_f`, where `M.m.p` represents the `crypto++`
version and `f` is a suffix number incremented each time a _patch tag_ is
created. _Patch tags_ will keep the same `crypto++` version as the main release
tag.

> As always, if you want to get the latest and greatest, always track the
> master branch.

## Standard usage

- Get this project using your favorite method (clone as submodule, get with
  [FetchContent](https://cmake.org/cmake/help/latest/module/FetchContent.html),
  get with [CPM](https://github.com/cpm-cmake/CPM.cmake)...)

- In your master CMakeLists.txt, add the following:

  ```cmake
  add_subdirectory(xxxx)
  # where xxx is the location where you put the cryptopp-cmake files
  ```

  That's pretty much it. You'll be able to link against `cryptopp` or the scoped
  alias `cryptopp::cryptopp` and have cmake handle everything else for you.

An example is located in the
[test/standard-cpm](https://github.com/abdes/cryptopp-cmake/tree/master/test)
directory.

## Using a local copy of crypto++

Certain users would prefer to have a fully disconnected project, and in such
scenario both the crypto++ source package and the cryptopp-cmake source package
would be pre-downloaded and then unpacked somewhere.

You would still need to add cryptopp-cmake as a subdirectory in your master
`CMakeLists.txt`, and you can set it up in such a way to use your local copy of
crypto++ via the option `CRYPTOPP_SOURCES`. Just set that option in the cmake
command line or in your CMakeLists.txt to point to the crypto++ source
directory. The rest will be taken care of for you.

## Requesting the master branch of cryptopp

If you want to test the bleeding edge of crypto++ with cmake, simply set the
option `CRYPTOPP_USE_MASTER_BRANCH` in your CMakeLists.txt or the cmake command
line and as usual, add the cryptopp-cmake as a subdirectory.

## Other ways

There are many other ways to use this project, including by directly picking the
files you need and adding them to your own project, by getting the package via
conan, etc... Take some time to read the source code, and make suggestions if
you need a new usage scenario via a new issue.

## Why Modern CMake?

Have a look at [Installing
CMake](https://cliutils.gitlab.io/modern-cmake/chapters/intro/installing.html)
from the online 'Modern CMake' book, to see a recent snapshot of which version
of CMake is being installed by default on Linux distributions.

![Packaging Status](https://repology.org/badge/vertical-allrepos/cmake.svg?columns=3&minversion=3.10.0)

And more than that, it's so easy to install a modern version of CMake on
Linux/MacOS/Windows, and many other OSes.

Looking at the release notes of CMake versions from 3.0 till now, a minimum
version requirement of
[3.21](https://cmake.org/cmake/help/latest/release/3.21.html) is a good starting
point. That release brings in particular presets and some nice quality of life
features that will make the maintenance and the use of this project much simpler
and pleasant. After all, there is no justification for doing free Open Source
without pleasure :smiley:

[build-matrix]: https://github.com/abdes/cryptopp-cmake/actions/workflows/cmake-build.yml
[build-status-master-badge]: https://github.com/abdes/cryptopp-cmake/actions/workflows/cmake-build.yml/badge.svg?branch=master
[commits]: https://github.com/abdes/cryptopp-cmake/commits
[last-commit-badge]: https://img.shields.io/github/last-commit/abdes/cryptopp-cmake
[latest-release]: https://github.com/abdes/cryptopp-cmake/releases/latest
[license-badge]: https://img.shields.io/github/license/abdes/cryptopp-cmake
[license]: https://opensource.org/licenses/BSD-3-Clause
[linux-badge]: https://img.shields.io/badge/OS-linux-blue
[macos-badge]: https://img.shields.io/badge/OS-macOS-blue
[release-badge]: https://img.shields.io/github/v/release/abdes/cryptopp-cmake
[windows-badge]: https://img.shields.io/badge/OS-windows-blue
