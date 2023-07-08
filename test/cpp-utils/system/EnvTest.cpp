#include <gtest/gtest.h>
#include <cpp-utils/system/env.h>
#include <string>

using std::string;

namespace {
	string read_env(const char* key) {
		const char* value = std::getenv(key);
		return value != nullptr ? string(value) : string();
	}
}

TEST(EnvTest, SetAndGetEnv_ValueIsCorrect) {
	cpputils::setenv("my_key", "my_value");
	EXPECT_EQ(string("my_value"), read_env("my_key"));
}

TEST(EnvTest, SetAndGetEnvWithSpacedValue_ValueIsCorrect) {
	cpputils::setenv("my_key", "my value with spaces");
	EXPECT_EQ(string("my value with spaces"), read_env("my_key"));
}

TEST(EnvTest, UnsetAndGetEnv_ValueIsEmpty) {
	cpputils::setenv("my_key", "my_value");
	cpputils::unsetenv("my_key");
	EXPECT_EQ(nullptr, std::getenv("my_key"));
}
