#if defined(_MSC_VER)
#include <Windows.h>
#include <VersionHelpers.h>
#endif

#include <iostream>
#include <cryfs/impl/CryfsException.h>
#include <cpp-utils/assert/backtrace.h>
#include "Cli.h"

using std::cerr;
using cryfs::ErrorCode;

int main(int argc, const char *argv[]) {
#if defined(_MSC_VER)
	if (!IsWindows7SP1OrGreater()) {
		std::cerr << "CryFS is currently only supported on Windows 7 SP1 (or later)." << std::endl;
		exit(1);
	}
#endif

	cpputils::showBacktraceOnCrash();

	try {
		cryfs_unmount::Cli().main(argc, argv);
	}
	catch (const cryfs::CryfsException &e) {
		if (e.what() != std::string()) {
			std::cerr << "Error " << static_cast<int>(e.errorCode()) << ": " << e.what() << std::endl;
		}
		return exitCode(e.errorCode());
	}
	catch (const std::runtime_error &e) {
		std::cerr << "Error: " << e.what() << std::endl;
		return exitCode(ErrorCode::UnspecifiedError);
	}
	return exitCode(ErrorCode::Success);
}
