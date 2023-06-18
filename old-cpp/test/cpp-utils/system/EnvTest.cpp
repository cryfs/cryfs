#include <gtest/gtest.h>
#include <cpp-utils/system/env.h>
#include <string>

using std::string;

TEST(EnvTest, SetAndGetEnv_ValueIsCorrect) {
	cpputils::setenv("my_key", "my_value");
	EXPECT_EQ(string("my_value"), string(std::getenv("my_key")));
}

TEST(EnvTest, SetAndGetEnvWithSpacedValue_ValueIsCorrect) {
	cpputils::setenv("my_key", "my value with spaces");
	EXPECT_EQ(string("my value with spaces"), string(std::getenv("my_key")));
}

TEST(EnvTest, UnsetAndGetEnv_ValueIsEmpty) {
	cpputils::setenv("my_key", "my_value");
	cpputils::unsetenv("my_key");
	EXPECT_EQ(nullptr, std::getenv("my_key"));
}
