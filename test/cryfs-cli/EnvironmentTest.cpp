#include <gtest/gtest.h>
#include <cryfs-cli/Environment.h>
#include <boost/optional.hpp>
#include <boost/filesystem.hpp>
#include <cpp-utils/system/env.h>

using namespace cryfs_cli;
using std::string;
using boost::optional;
using boost::none;

#if defined(_MSC_VER)
constexpr const char* some_local_state_dir = "C:/my/local/state/dir";
#else
constexpr const char* some_local_state_dir = "/my/local/state/dir";
#endif

namespace bf = boost::filesystem;

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
            cpputils::setenv(key.c_str(), value.c_str());
        }
        ~WithEnv() {
            if (none == _oldValue) {
                cpputils::unsetenv(_key.c_str());
            } else {
                cpputils::setenv(_key.c_str(), _oldValue->c_str());
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

TEST_F(EnvironmentTest, LocalStateDir_NotSet) {
    EXPECT_EQ(Environment::defaultLocalStateDir(), Environment::localStateDir());
}

TEST_F(EnvironmentTest, LocalStateDir_Set) {
    WithEnv env("CRYFS_LOCAL_STATE_DIR", some_local_state_dir);
    EXPECT_EQ(some_local_state_dir, Environment::localStateDir().string());
}

TEST_F(EnvironmentTest, LocalStateDir_ConvertsRelativeToAbsolutePath_WithDot) {
    WithEnv env("CRYFS_LOCAL_STATE_DIR", "./dir");
    EXPECT_EQ((bf::current_path() / "./dir").string(), Environment::localStateDir().string());
}

TEST_F(EnvironmentTest, LocalStateDir_ConvertsRelativeToAbsolutePath_WithoutDot) {
    WithEnv env("CRYFS_LOCAL_STATE_DIR", "dir");
    EXPECT_EQ((bf::current_path() / "dir").string(), Environment::localStateDir().string());
}
