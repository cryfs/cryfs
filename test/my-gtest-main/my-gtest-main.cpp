#include "my-gtest-main.h"

#include "gmock/gmock.h"
#include "gtest/gtest.h"

#include <boost/optional.hpp>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/assert/backtrace.h>

namespace {
	// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
	boost::optional<boost::filesystem::path> executable;
}

const boost::filesystem::path& get_executable() {
	ASSERT(executable != boost::none, "Executable path not set");
	return *executable;
}


int main(int argc, char** argv) {
	executable = boost::filesystem::path(argv[0]);

	// Print a backtrace if a test crashes, otherwise a crashing test binary tells us nothing.
	// This only handles the actual crash signals and not SIGABRT, because ASSERT() and gtest
	// death tests use SIGABRT deliberately, and it doesn't swallow the signal, so a crashing
	// test binary still reports as killed-by-signal and still dumps core.
	cpputils::showBacktraceOnCrashSignals();

	// Since Google Mock depends on Google Test, InitGoogleMock() is
	// also responsible for initializing Google Test.  Therefore there's
	// no need for calling testing::InitGoogleTest() separately.
	testing::InitGoogleMock(&argc, argv);
	return RUN_ALL_TESTS();
}
