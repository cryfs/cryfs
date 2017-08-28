#!/bin/sh

### Constants
c_valgrind_min=1
reference_file="${scriptdir}/test_scrypt.good"
encrypted_reference_file="${scriptdir}/test_scrypt_good.enc"
decrypted_reference_file="${out}/attempt_reference.txt"

scenario_cmd() {
	# Decrypt a reference file.
	setup_check_variables
	(
		echo ${password} | ${c_valgrind_cmd} ${bindir}/scrypt	\
		    dec -P ${encrypted_reference_file}			\
		    ${decrypted_reference_file}
		echo $? > ${c_exitfile}
	)

	# The decrypted reference file should match the reference.
	setup_check_variables
	if cmp -s ${decrypted_reference_file} ${reference_file}; then
		echo "0"
	else
		echo "1"
	fi > ${c_exitfile}
}
