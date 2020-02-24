#!/bin/bash

# Note: Call this from a cmake build directory (e.g. cmake/) for out-of-source builds
# Examples:
# mkdir cmake && cd cmake && ../run-clang-tidy.sh
# mkdir cmake && cd cmake && ../run-clang-tidy.sh -fix
# mkdir cmake && cd cmake && ../run-clang-tidy.sh -export-fixes fixes.yaml

set -e

export NUMCORES=`nproc` && if [ ! -n "$NUMCORES" ]; then export NUMCORES=`sysctl -n hw.ncpu`; fi
echo Using ${NUMCORES} cores

# Run cmake in current working directory, but on source that is in the same directory as this script file
cmake -DBUILD_TESTING=on -DCMAKE_EXPORT_COMPILE_COMMANDS=ON "${0%/*}"

# Filter all third party code from the compilation database
cat compile_commands.json|jq "map(select(.file | test(\"^$(realpath ${0%/*})/(src|test)/.*$\")))" > compile_commands2.json
rm compile_commands.json
mv compile_commands2.json compile_commands.json

run-clang-tidy.py -j${NUMCORES} -quiet -header-filter "$(realpath ${0%/*})/(src|test)/.*" $@
