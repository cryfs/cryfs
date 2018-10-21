#!/bin/bash

# Note: Call this from a cmake build directory (e.g. cmake/) for out-of-source builds
# Examples:
# mkdir cmake && cd cmake && ../run-iwqu.sh
# mkdir cmake && cd cmake && ../run-iwqu.sh -fix

set -e

export NUMCORES=`nproc` && if [ ! -n "$NUMCORES" ]; then export NUMCORES=`sysctl -n hw.ncpu`; fi
echo Using ${NUMCORES} cores

# Run cmake in current working directory, but on source that is in the same directory as this script file
cmake -DBUILD_TESTING=on -DCMAKE_EXPORT_COMPILE_COMMANDS=ON "${0%/*}"

# Filter all third party code from the compilation database
cat compile_commands.json|jq "map(select(.file | test(\"^$(realpath ${0%/*})/(src|test)/.*$\")))" > compile_commands2.json
rm compile_commands.json
mv compile_commands2.json compile_commands.json

if [ "$1" = "-fix" ]; then
  TMPFILE=/tmp/iwyu.`cat /dev/urandom | tr -cd 'a-f0-9' | head -c 8`.out

  function cleanup {
    rm ${TMPFILE}
  }
  trap cleanup EXIT

  iwyu_tool -j${NUMCORES} -p. ${@:2} | tee ${TMPFILE}
  fix_include < ${TMPFILE}
else
  iwyu_tool -j${NUMCORES} -p. $@
fi
