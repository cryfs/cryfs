#!/bin/bash

set -e

# Workaround homebrew bug, see https://twitter.com/axccl/status/1083393735277363205 and https://github.com/openPMD/openPMD-api/pull/431/files
#brew upgrade --cleanup || brew upgrade --cleanup

# Install newer GCC if we're running on GCC
if [ "${CXX}" == "g++" ]; then
    # We need to uninstall oclint because it creates a /usr/local/include/c++ symlink that clashes with the gcc5 package
    # see https://github.com/Homebrew/homebrew-core/issues/21172
    brew cask uninstall oclint
    brew install gcc@7
fi

brew cask install osxfuse
brew install libomp

# By default, travis only fetches the newest 50 commits. We need more in case we're further from the last version tag, so the build doesn't fail because it can't generate the version number.
git fetch --unshallow --tags

# Setup ccache
brew install ccache
