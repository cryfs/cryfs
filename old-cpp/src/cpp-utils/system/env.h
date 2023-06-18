#pragma once
#ifndef MESSMER_CPPUTILS_SYSTEM_ENV_H
#define MESSMER_CPPUTILS_SYSTEM_ENV_H

namespace cpputils {
void setenv(const char* key, const char* value);
void unsetenv(const char* key);
}

#endif
