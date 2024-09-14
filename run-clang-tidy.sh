#!/bin/bash

# Note: Call this from a cmake build directory (e.g. cmake/) for out-of-source builds
# Examples:
# mkdir cmake && cd cmake && ../run-clang-tidy.sh
# mkdir cmake && cd cmake && ../run-clang-tidy.sh -fix
# mkdir cmake && cd cmake && ../run-clang-tidy.sh -export-fixes fixes.yaml

set -e
set -v

SCRIPT=run-clang-tidy-16.py

export NUMCORES=`nproc` && if [ ! -n "$NUMCORES" ]; then export NUMCORES=`sysctl -n hw.ncpu`; fi
echo Using ${NUMCORES} cores

# Run cmake in current working directory, but on source that is in the same directory as this script file
conan build . --build=missing -o "&:build_tests=True" -o "&:export_compile_commands=True" -o "&:use_ccache=True" -s build_type=Debug

# Filter all third party code from the compilation database
ROOTPATH=$(realpath ${0%/*})
cd build/Debug
cat compile_commands.json|jq "map(select(.file | test(\"^${ROOTPATH}/(src|test)/.*$\")))" > compile_commands2.json
rm compile_commands.json
mv compile_commands2.json compile_commands.json

${SCRIPT} -j${NUMCORES} -quiet -config-file ../../.clang-tidy -header-filter "${ROOTPATH}/(src|test)/.*" $@
