# CryFS

CryFS encrypts your files, so you can safely store them anywhere. It works well together with cloud services like Dropbox, iCloud, OneDrive and others.
See [https://www.cryfs.org](https://www.cryfs.org).

Install latest release
======================

Linux
------

CryFS is available through apt, but depending on which version of Ubuntu or Debian you're using, you may get an old version.

    sudo apt install cryfs

The following should work on Arch and Arch-based distros:

    sudo pacman -S cryfs

If you use homebrew-core, using the following instruction you should be able to install CrysFS:

    brew install cryfs/tap/cryfs

Additionally, the following would work for any Linux distro with the Nix package manager:

    nix-env -iA nixpkgs.cryfs

OSX
----

CryFS is distributed via Homebrew, MacPorts, and Nix.

If you use Homebrew:

    brew install --cask macfuse
    brew install cryfs/tap/cryfs

If you use MacPorts:

    port install cryfs

For Nix, the macOS build for cryfs is available in the Nixpkgs channel 21.05
and later:

    brew install --cask macfuse # or download from https://osxfuse.github.io/
    nix-env -iA nixpkgs.cryfs

Windows (experimental)
----------------------

CryFS has experimental Windows support since the 0.10 release series. To install it, do:

1. Install [DokanY](https://github.com/dokan-dev/dokany/releases)
2. Install [Microsoft Visual C++ Redistributable for Visual Studio 2019](https://support.microsoft.com/en-us/help/2977003/the-latest-supported-visual-c-downloads)
3. Install [CryFS](https://www.cryfs.org/#download)

GUI
===
There are some GUI applications with CryFS support. You usually have to install the GUI **and** also CryFS itself for it to work.
- [SiriKali](https://mhogomchungu.github.io/sirikali/)
- [Plasma Vault](https://www.kde.org/announcements/plasma-5.11.0.php) in KDE Plasma >= 5.11

Stability / Production readiness
====================
CryFS 0.10 or later is stable for most use cases, but has a couple of known issues that can corrupt your file system.
They don't happen in normal day to day use, but can happen if you don't pay attention or aren't aware of them.
This is why the version number hasn't reached 1.0 yet.

- If you kill the CryFS process while it was in the middle of writing data (either intentionally or unintentionally by losing power to your PC), your file system could get corrupted.
  CryFS does not do journaling. Note that in 0.10.x, read accesses into a CryFS file system can cause writes because file timestamps get updated. So if you're unlucky, your file system
  could get corrupted if you lose power while you were reading files as well. Read accesses aren't an issue in CryFS 0.11.x anymore, because it mounts the filesystem with `noatime` by default.
- The same corruption mentioned above can happen when CryFS is trying to write data but your disk ran out of space, causing the write to fail.
- CryFS does not currently support concurrent access, i.e. accessing a file system from multiple devices at the same time.
  CryFS works very well for storing data in a cloud and using it from multiple devices, but you need to make sure that only one CryFS process is active at any point in time, and you also need
  to make sure that the cloud synchronization client (e.g. Dropbox) finishes its synchronization before you switch devices. There are some ideas on how concurrent access could be supported in
  future versions, but it's a hard problem to solve. If you do happen to access the file system from multiple devices at the same time, it will likely go well most of the time, but it can corrupt your file system.
- In addition to the scenarios above that can corrupt your file system, note that there is currently no fsck-like tool for CryFS that could recover your data. Although such a tool is in theory, possible,
  it hasn't been implemented yet and a corrupted file system will most likely cause a loss of your data.

If the scenarios mentioned above don't apply to you, then you can consider CryFS 0.10.x and later as stable. The 0.9.x versions are not recommended anymore.

Building from source
====================

Requirements
------------
  - Git (for getting the source code)
  - GCC version >= 7 or Clang >= 7
  - CMake version >= 3.25
  - pkg-config (on Unix)
  - Conan package manager (version 2.x)
  - libFUSE version >= 2.9 (including development headers), on Mac OS X instead install macFUSE from https://osxfuse.github.io/
  - Python >= 3.5
  - OpenMP

You can use the following commands to install these requirements

    # Ubuntu
    $ sudo apt install git python3 g++ cmake libomp-dev pkg-config libfuse-dev fuse

    # Fedora
    $ sudo dnf install git python3 gcc-c++ cmake pkgconf fuse-devel perl

    # Macintosh
    # TODO Update the package list
    $ brew install cmake pkg-config libomp macfuse

To install conan, follow the [official installation instructions](https://docs.conan.io/2/installation.html). The following steps should work on Ubuntu/Debian based systems:

    $ sudo apt install pipx
    $ pipx install conan~=2.7.0
    $ pipx ensurepath

Restart your shell so that conan is on your PATH, and then let it find your compiler

    $ conan profile detect

You can edit the generated profile file (usually `~/.conan2/profiles/default`) if you want to use different compiler settings.


Build & Install
---------------
See further below in this README for instructions on how to build a .deb/.rpm package instead of installing CryFS directly.

 1. Clone repository

        $ git clone https://github.com/cryfs/cryfs.git cryfs
        $ cd cryfs

 2. Build

        $ conan build . -s build_type=RelWithDebInfo --build=missing
        
    The executable will be generated at `build/RelWithDebInfo/src/cryfs-cli/cryfs`

 3. Install

        $ cd build/RelWithDebInfo
        $ sudo make install

You can pass the following build types to the *conan build* command (using *-s build_type=value*):
 - **Debug**: No optimizations, debug symbols enabled, assertions enabled
 - **RelWithDebInfo**: Optimizations enabled, debug symbols enabled, assertions enabled
 - **Release**: Optimizations enabled, no debug symbols, no assertions

You can pass the following options to the *conan build* command (using *-o "&:key=value"*):
 - **build_tests**=[True|False]: Whether to build the test cases (can take a long time). Default: False.
 - **update_checks**=[True|False]: Build a CryFS that doesn't check online for updates and security vulnerabilities. Default: True.
 - **disable_openmp**=[True|False]: Disable OpenMP support. Default: False.


Run tests
---------
Follow the build & install steps from above, but add the `-o "&:build_tests=True"` parameter to conan:

    $ conan build . -s build_type=RelWithDebInfo --build=missing -s build_type=Debug -o "&:build_tests=True"

Then run the tests:

    $ cd build/Debug/test
    $ ./blobstore/blobstore-test
    $ ./blockstore/blockstore-test
    $ ./cpp-utils/cpp-utils-test
    $ ./cryfs/cryfs-test
    $ ./cryfs-cli/cryfs-cli-test
    $ ./fspp/fspp-test
    $ ./gitversion/gitversion-test
    $ ./parallelaccessstore/parallelaccessstore-test

Building on Windows (experimental)
----------------------------------
1. Install conan 2. If you want to use "pip install conan", you may have to install Python first.
2. Install DokanY 2.2.0.1000. Other versions may not work.
3. Build the project

        $ conan build . --build=missing -o "&:windows_dokany_path=C:/Program Files/Dokan/DokanLibrary-2.2.0"

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

To use local dependencies, you can install all of CryFS's dependencies (e.g. boost, spdlog) manually and run cmake directly without invoking conan first:

    $ mkdir build
    $ cd build
    $ cmake ..
    $ make

It is recommended to use the same versions of the dependencies as stated in the conanfile.py in this repository.
It might be useful to take a look at [how our CI setup installs those dependencies](https://github.com/cryfs/cryfs/blob/develop/.github/workflows/actions/install_local_dependencies/action.yaml) to get you started.

CMake will use pkg-config to find those dependencies.

Creating .deb and .rpm packages
-------------------------------

It is recommended to install CryFS using packages, because that allows for an easy way to uninstall it again once you don't need it anymore.

If you want to create a .rpm package, you need to install rpmbuild.

 1. Clone repository

        $ git clone https://github.com/cryfs/cryfs.git cryfs
        $ cd cryfs

 2. Make sure you have the required dependencies

        $ sudo apt install file dpkg-dev rpm

 3. Build

        $ conan build . -s build_type=RelWithDebInfo --build=missing
        $ cd build/RelWithDebInfo
        $ make package

Disclaimer
----------------------

In the event of a password leak, you are strongly advised to create a new filesystem and copy all the data over from the previous one. Then, remove all copies of the compromised filesystem and config file(e.g, from the "previous versions" feature of your cloud system) to prevent access to the key (and, as a result, your data) using the leaked password.
