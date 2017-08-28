#!/bin/sh

### Definitions
#
# This test suite uses the following terminology:
# - scenario: a series of commands to test.  Each must be in a
#       separate file, and must be completely self-contained
#       (other than the variables listed below).
# - check: a series of commands that produces an exit code which
#       the test suite should check.  A scenario may contain any
#       number of checks.
#
### Design
#
# The main function is scenario_runner(scenario_filename), which
# takes a scenario file as the argument, and runs a
#     scenario_cmd()
# function which was defined in that file.
#
### Variables
#
# Wherever possible, this suite uses local variables and
# explicitly-passed arguments, with the following exceptions:
# - s_basename: this is the basename for the scenario's temporary
#       and log files.
# - s_val_basename: this is the basename for the scenario's
#       valgrind log files.
# - s_count: this is the count of the scenario's checks (so that
#       each check can have distinct files).
# - s_retval: this is the overall exit code of the scenario.
# - c_exitfile: this contains the exit code of each check.
# - c_valgrind_min: this is the minimum value of USE_VALGRIND
#       which will enable valgrind checking for this check.
# - c_valgrind_cmd: this is the valgrind command (including
#       appropriate log file) if necessary, or is "" otherwise.

set -o nounset

### Constants
out="tests-output"
out_valgrind="tests-valgrind"
valgrind_suppressions="${out_valgrind}/suppressions"
valgrind_suppressions_log="${out_valgrind}/suppressions.pre"

# Keep the user-specified ${USE_VALGRIND}, or initialize to 0.
USE_VALGRIND=${USE_VALGRIND:-0}

# A non-zero value unlikely to be used as an exit code by the programs being
# tested.
valgrind_exit_code=108

## prepare_directories():
# Delete any old directories, and create new ones as necessary.  Must be run
# after check_optional_valgrind().
prepare_directories() {
	# Clean up previous directories.
	if [ -d "${out}" ]; then
		rm -rf ${out}
	fi
	if [ -d "${out_valgrind}" ]; then
		rm -rf ${out_valgrind}
	fi

	# Make new directories.
	mkdir ${out}
	if [ "$USE_VALGRIND" -gt 0 ]; then
		mkdir ${out_valgrind}
	fi
}

## find_system (cmd, args...):
# Looks for ${cmd} in the $PATH, and ensure that it supports ${args}.
find_system() {
	cmd=$1
	cmd_with_args=$@
	# Look for ${cmd}.
	system_binary=`command -v ${cmd}`
	if [ -z "${system_binary}" ]; then
		system_binary=""
		printf "System ${cmd} not found.\n" 1>&2
	# If the command exists, check it ensures the ${args}.
	elif ${cmd_with_args} 2>&1 >/dev/null |	\
	    grep -qE "(invalid|illegal) option"; then
		system_binary=""
		printf "Cannot use system ${cmd}; does not" 1>&2
		printf " support necessary arguments.\n" 1>&2
	fi
	echo "${system_binary}"
}

## check_optional_valgrind ():
# Return a $USE_VALGRIND variable defined; if it was previously defined and
# was greater than 0, then check that valgrind is available in the $PATH.
check_optional_valgrind() {
	if [ "$USE_VALGRIND" -gt 0 ]; then
		# Look for valgrind in $PATH.
		if ! command -v valgrind >/dev/null 2>&1; then
			printf "valgrind not found\n" 1>&2
			exit 1
		fi
	fi
}

## ensure_valgrind_suppresssion (potential_memleaks_binary):
# Runs the ${potential_memleaks_binary} through valgrind, keeping
# track of any apparent memory leak in order to suppress reporting
# those leaks when testing other binaries.
ensure_valgrind_suppression() {
	potential_memleaks_binary=$1

	# Quit if we're not using valgrind.
	if [ ! "$USE_VALGRIND" -gt 0 ]; then
		return
	fi;
	printf "Generating valgrind suppressions... "

	# Run valgrind on the binary, sending it a "\n" so that
	# a test which uses STDIN will not wait for user input.
	printf "\n" | (valgrind --leak-check=full --show-leak-kinds=all	\
		--gen-suppressions=all					\
		--log-file=${valgrind_suppressions_log}			\
		${potential_memleaks_binary})

	# Strip out useless parts from the log file, as well as
	# removing references to the main and "pl_*" ("potential
	# loss") functions so that the suppressions can apply to
	# other binaries.
	(grep -v "^==" ${valgrind_suppressions_log} 			\
		| grep -v "   fun:pl_" -				\
		| grep -v "   fun:main" -				\
		> ${valgrind_suppressions} )

	# Clean up
	rm -f ${valgrind_suppressions_log}
	printf "done.\n"
}

## setup_check_variables ():
# Set up the "check" variables ${c_exitfile} and ${c_valgrind_cmd}, the
# latter depending on the previously-defined ${c_valgrind_min}.
# Advances the number of checks ${s_count} so that the next call to this
# function will set up new filenames.
setup_check_variables() {
	# Set up the "exit" file.
	c_exitfile="${s_basename}-`printf %02d ${s_count}`.exit"

	# If we don't have a suppressions file, don't try to use it.
	if [ ! -e ${valgrind_suppressions} ]; then
		valgrind_suppressions=/dev/null
	fi

	# Set up the valgrind command if $USE_VALGRIND is greater
	# than or equal to ${valgrind_min}; otherwise, produce an
	# empty string.  Using --error-exitcode means that if
	# there is a serious problem (such that scrypt calls
	# exit(1)) *and* a memory leak, the test suite reports an
	# exit value of ${valgrind_exit_code}.  However, if there
	# is a serious problem but no memory leak, we still
	# receive a non-zero exit code.  The most important thing
	# is that we only receive an exit code of 0 if both the
	# program and valgrind are happy.
	if [ "$USE_VALGRIND" -ge "${c_valgrind_min}" ]; then
		val_logfilename=${s_val_basename}-`printf %02d ${s_count}`.log
		c_valgrind_cmd="valgrind \
			--log-file=${val_logfilename} \
			--leak-check=full --show-leak-kinds=all \
			--errors-for-leak-kinds=all \
			--suppressions=${valgrind_suppressions} \
			--error-exitcode=${valgrind_exit_code} "
	else
		c_valgrind_cmd=""
	fi

	# Advances the number of checks.
	s_count=$((s_count + 1))
}

## get_val_logfile (val_basename, exitfile):
# Return the valgrind logfile corresponding to ${exitfile}.
get_val_logfile() {
	val_basename=$1
	exitfile=$2
	num=`echo "${exitfile}" | rev | cut -c 1-7 | rev | cut -c 1-2 `
	echo "${val_basename}-${num}.log"
}

## notify_success_or_fail (log_basename, val_log_basename):
# Examine all "exit code" files beginning with ${log_basename} and
# print "SUCCESS!" or "FAILED!" as appropriate.  If the test failed
# with the code ${valgrind_exit_code}, output the appropriate
# valgrind logfile to stdout.
notify_success_or_fail() {
	log_basename=$1
	val_log_basename=$2

	# Check each exitfile.
	for exitfile in `ls ${log_basename}-*.exit | sort`; do
		ret=`cat ${exitfile}`
		if [ "${ret}" -lt 0 ]; then
			echo "SKIP!"
			return
		fi
		if [ "${ret}" -gt 0 ]; then
			echo "FAILED!"
			retval=${ret}
			if [ "${ret}" -eq "${valgrind_exit_code}" ]; then
				val_logfilename=$( get_val_logfile \
					${val_log_basename} ${exitfile} )
				cat ${val_logfilename}
			fi
			s_retval=${ret}
			return
		fi
	done

	echo "SUCCESS!"
}

## scenario_runner (scenario_filename):
# Runs a test scenario from ${scenario_filename}.
scenario_runner() {
	scenario_filename=$1
	basename=`basename ${scenario_filename} .sh`
	printf "  ${basename}... " 1>&2

	# Initialize "scenario" and "check" variables.
	s_basename=${out}/${basename}
	s_val_basename=${out_valgrind}/${basename}
	s_count=0
	c_exitfile=/dev/null
	c_valgrind_min=9
	c_valgrind_cmd=""

	# Load scenario_cmd() from the scenario file.
	unset scenario_cmd
	. ${scenario_filename}
	if ! command -v scenario_cmd 1>/dev/null ; then
		printf "ERROR: scenario_cmd() is not defined in\n"
		printf "  ${scenario_filename}\n"
		exit 1
	fi

	# Run the scenario command.
	scenario_cmd

	# Print PASS or FAIL, and return result.
	s_retval=0
	notify_success_or_fail ${s_basename} ${s_val_basename}

	return "${s_retval}"
}

## run_scenarios (scenario_filenames):
# Runs all scenarios matching ${scenario_filenames}.
run_scenarios() {
	printf -- "Running tests\n"
	printf -- "-------------\n"
	scenario_filenames=$@
	for scenario in ${scenario_filenames}; do
		# We can't call this function with $( ... ) because we
		# want to allow it to echo values to stdout.
		scenario_runner ${scenario}
		retval=$?
		if [ ${retval} -gt 0 ]; then
			exit ${retval}
		fi
	done
}
