#!/bin/bash

# Note: Call this from a cmake build directory (e.g. cmake/) for out-of-source builds
# Examples:
# mkdir cmake && cd cmake && ../run-clang-tidy.sh
# mkdir cmake && cd cmake && ../run-clang-tidy.sh -fix
# mkdir cmake && cd cmake && ../run-clang-tidy.sh -export-fixes fixes.yaml

set -e

NUMCORES=`nproc`

# Run cmake in current working directory, but on source that is in the same directory as this script file
cmake -DBUILD_TESTING=on -DCMAKE_EXPORT_COMPILE_COMMANDS=ON "${0%/*}"

# Build scrypt first. Our Makefiles call ino theirs, and this is needed to generate some header files. Clang-tidy will otherwise complain they're missing.
make -j${NUMCORES} scrypt

run-clang-tidy.py -j${NUMCORES} -quiet -header-filter "$(realpath ${0%/*})/(src|test)/.*" $@

