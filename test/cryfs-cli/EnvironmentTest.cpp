#include <gtest/gtest.h>
#include <cryfs-cli/Environment.h>
#include <boost/optional.hpp>

using namespace cryfs;
using std::string;
using boost::optional;
using boost::none;

class EnvironmentTest : public ::testing::Test {
public:
    // WithEnv sets an environment variable while it is in scope.
    // Once it leaves scope, the environment is reset.
    class WithEnv {
    public:
        WithEnv(const string &key, const string &value): _key(key) , _oldValue(none) {
            char *oldValue = std::getenv(key.c_str());
            if (nullptr != oldValue) {
                _oldValue = string(oldValue);
            }
            ::setenv(key.c_str(), value.c_str(), 1);
        }
        ~WithEnv() {
            if (none == _oldValue) {
                ::unsetenv(_key.c_str());
            } else {
                ::setenv(_key.c_str(), _oldValue->c_str(), 1);
            }
        }

    private:
        string _key;
        optional<string> _oldValue;
    };
};

TEST_F(EnvironmentTest, Noninteractive_Unset) {
    EXPECT_FALSE(Environment::isNoninteractive());
}

TEST_F(EnvironmentTest, Noninteractive_Set) {
    WithEnv env("CRYFS_FRONTEND", "noninteractive");
    EXPECT_TRUE(Environment::isNoninteractive());
}

TEST_F(EnvironmentTest, Noninteractive_SetToOtherValue) {
    WithEnv env("CRYFS_FRONTEND", "someotherfrontend");
    EXPECT_FALSE(Environment::isNoninteractive());
}

TEST_F(EnvironmentTest, NoUpdateCheck_Unset) {
    EXPECT_FALSE(Environment::noUpdateCheck());
}

TEST_F(EnvironmentTest, NoUpdateCheck_Set) {
    WithEnv env("CRYFS_NO_UPDATE_CHECK", "true");
    EXPECT_TRUE(Environment::noUpdateCheck());
}

TEST_F(EnvironmentTest, NoUpdateCheck_SetToOtherValue) {
    WithEnv env("CRYFS_NO_UPDATE_CHECK", "someothervalue");
    // No matter what the value is, setting the environment variable says we don't do update checks.
    EXPECT_TRUE(Environment::noUpdateCheck());
}
