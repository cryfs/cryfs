#include "assert.h"

thread_local int cpputils::_assert::DisableAbortOnFailedAssertionRAII::num_instances_ = 0;
