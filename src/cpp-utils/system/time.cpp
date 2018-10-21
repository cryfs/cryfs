#include "time.h"
#include <chrono>

using std::chrono::system_clock;
using std::chrono::duration_cast;
using std::chrono::seconds;
using std::chrono::nanoseconds;

namespace cpputils {
namespace time {

struct timespec now() {
	auto now = system_clock::now().time_since_epoch();
	struct timespec spec{};
	spec.tv_sec = duration_cast<seconds>(now).count();
	spec.tv_nsec = duration_cast<nanoseconds>(now).count() % 1000000000;
	return spec;
}

}
}
