#!/bin/sh

### Constants
c_valgrind_min=1
reference_file="${scriptdir}/test_scrypt.good"
encrypted_file="${out}/attempt.enc"
decrypted_file="${out}/attempt.txt"

scenario_cmd() {
	# Encrypt a file.
	setup_check_variables
	(
		echo ${password} | ${c_valgrind_cmd} ${bindir}/scrypt	\
		    enc -P -t 1 ${reference_file} ${encrypted_file}
		echo $? > ${c_exitfile}
	)

	# The encrypted file should be different from the original file.
	# We cannot check against the "reference" encrypted file, because
	# encrypted files include random salt.  If successful, don't delete
	# ${encrypted_file} yet; we need it for the next test.
	setup_check_variables
	if cmp -s ${encrypted_file} ${reference_file}; then
		echo "1"
	else
		echo "0"
	fi > ${c_exitfile}

	# Decrypt the file we just encrypted.
	setup_check_variables
	(
		echo ${password} | ${c_valgrind_cmd} ${bindir}/scrypt	\
		    dec -P ${encrypted_file} ${decrypted_file}
		echo $? > ${c_exitfile}
	)

	# The decrypted file should match the reference.
	setup_check_variables
	if cmp -s ${decrypted_file} ${reference_file}; then
		echo "0"
	else
		echo "1"
	fi > ${c_exitfile}
}
