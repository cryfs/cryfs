#!/bin/sh

# Build directory (allowing flexible out-of-tree builds).
bindir=$1

# Constants used in multiple scenarios.
password="hunter2"

# Find script directory and load helper functions.
scriptdir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P)
. ${scriptdir}/shared_test_functions.sh

# We need a ${bindir}.
if [ -z ${bindir} ]; then
	printf "Warning: Scrypt binary directory not given.\n"
	printf "Attempting to use default values for in-source-tree build.\n"
	bindir=".."
fi

# Find system scrypt, and ensure it supports -P.
system_scrypt=$( find_system scrypt enc -P )

# Check for optional valgrind.
check_optional_valgrind

# Clean up previous directories, and create new ones.
prepare_directories

# Generate valgrind suppression file if it is required.  Must be
# done after preparing directories.
ensure_valgrind_suppression ${bindir}/tests/valgrind/potential-memleaks

# Run the test scenarios; this will exit on the first failure.
run_scenarios ${scriptdir}/??-*.sh
