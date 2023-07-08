#include "testutils/ProgramOptionsTestBase.h"
#include <cryfs-cli/program_options/utils.h>

using namespace cryfs_cli::program_options;
using std::pair;
using std::vector;
using std::string;

class ProgramOptionsUtilsTest: public ProgramOptionsTestBase {};

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_ZeroOptions) {
    const vector<string> input = {"./executableName"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneShortOption) {
    const vector<string> input = {"./executableName", "-j"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "-j"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneLongOption) {
    const vector<string> input = {"./executableName", "--myoption"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "--myoption"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OnePositionalOption) {
    const vector<string> input = {"./executableName", "mypositionaloption"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneShortOption_DoubleDash) {
    const vector<string> input = {"./executableName", "-j", "--"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "-j"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneLongOption_DoubleDash) {
    const vector<string> input = {"./executableName", "--myoption", "--"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "--myoption"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OnePositionalOption_DoubleDash) {
    const vector<string> input = {"./executableName", "mypositionaloption", "--"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_DoubleDash_OneShortOption) {
    const vector<string> input = {"./executableName", "--", "-a"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({"-a"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_DoubleDash_OneLongOption) {
    const vector<string> input = {"./executableName", "--", "--myoption"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({"--myoption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_DoubleDash_OnePositionalOption) {
    const vector<string> input = {"./executableName", "--", "mypositionaloption"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({"mypositionaloption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneShortOption_DoubleDash_OneShortOption) {
    const vector<string> input = {"./executableName", "-j", "--", "-a"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "-j"}, result.first);
    EXPECT_VECTOR_EQ({"-a"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneLongOption_DoubleDash_OneLongOption) {
    const vector<string> input = {"./executableName", "--myoption", "--", "--myotheroption"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "--myoption"}, result.first);
    EXPECT_VECTOR_EQ({"--myotheroption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OnePositionalOption_DoubleDash_OnePositionalOption) {
    const vector<string> input = {"./executableName", "mypositionaloption", "--", "otherpositionaloption"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption"}, result.first);
    EXPECT_VECTOR_EQ({"otherpositionaloption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_MoreOptions) {
    const vector<string> input = {"./executableName", "mypositionaloption", "myotherpositionaloption", "-j", "--alpha", "--", "filename", "--beta", "-j3"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption", "myotherpositionaloption", "-j", "--alpha"}, result.first);
    EXPECT_VECTOR_EQ({"filename", "--beta", "-j3"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_RealisticCryfsOptions) {
    const vector<string> input = {"./executableName", "rootDir", "mountDir", "--", "-f"};
    const pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "rootDir", "mountDir"}, result.first);
    EXPECT_VECTOR_EQ({"-f"}, result.second);
}
