#include "testutils/ProgramOptionsTestBase.h"
#include <cryfs-cli/program_options/utils.h>

using namespace cryfs_cli::program_options;
using std::pair;
using std::vector;
using std::string;

class ProgramOptionsUtilsTest: public ProgramOptionsTestBase {};

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_ZeroOptions) {
    vector<string> input = {"./executableName"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneShortOption) {
    vector<string> input = {"./executableName", "-j"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "-j"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneLongOption) {
    vector<string> input = {"./executableName", "--myoption"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "--myoption"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OnePositionalOption) {
    vector<string> input = {"./executableName", "mypositionaloption"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneShortOption_DoubleDash) {
    vector<string> input = {"./executableName", "-j", "--"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "-j"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneLongOption_DoubleDash) {
    vector<string> input = {"./executableName", "--myoption", "--"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "--myoption"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OnePositionalOption_DoubleDash) {
    vector<string> input = {"./executableName", "mypositionaloption", "--"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption"}, result.first);
    EXPECT_VECTOR_EQ({}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_DoubleDash_OneShortOption) {
    vector<string> input = {"./executableName", "--", "-a"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({"-a"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_DoubleDash_OneLongOption) {
    vector<string> input = {"./executableName", "--", "--myoption"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({"--myoption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_DoubleDash_OnePositionalOption) {
    vector<string> input = {"./executableName", "--", "mypositionaloption"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({"mypositionaloption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneShortOption_DoubleDash_OneShortOption) {
    vector<string> input = {"./executableName", "-j", "--", "-a"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "-j"}, result.first);
    EXPECT_VECTOR_EQ({"-a"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneLongOption_DoubleDash_OneLongOption) {
    vector<string> input = {"./executableName", "--myoption", "--", "--myotheroption"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "--myoption"}, result.first);
    EXPECT_VECTOR_EQ({"--myotheroption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OnePositionalOption_DoubleDash_OnePositionalOption) {
    vector<string> input = {"./executableName", "mypositionaloption", "--", "otherpositionaloption"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption"}, result.first);
    EXPECT_VECTOR_EQ({"otherpositionaloption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_MoreOptions) {
    vector<string> input = {"./executableName", "mypositionaloption", "myotherpositionaloption", "-j", "--alpha", "--", "filename", "--beta", "-j3"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption", "myotherpositionaloption", "-j", "--alpha"}, result.first);
    EXPECT_VECTOR_EQ({"filename", "--beta", "-j3"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_RealisticCryfsOptions) {
    vector<string> input = {"./executableName", "rootDir", "mountDir", "--", "-f"};
    pair<vector<string>,vector<string>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "rootDir", "mountDir"}, result.first);
    EXPECT_VECTOR_EQ({"-f"}, result.second);
}
