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

TEST_F(ProgramOptionsUtilsTest, ExtractNonemptyOption_NoOptions) {
    vector<string> input = {};
    EXPECT_FALSE(extractNonemptyOption(&input));
    EXPECT_VECTOR_EQ({}, input);
}

TEST_F(ProgramOptionsUtilsTest, ExtractNonemptyOption_NotThere) {
    vector<string> input = {"-o", "allow_other", "-o", "noatime"};
    EXPECT_FALSE(extractNonemptyOption(&input));
    EXPECT_VECTOR_EQ({"-o", "allow_other", "-o", "noatime"}, input);
}

TEST_F(ProgramOptionsUtilsTest, ExtractNonemptyOption_OnlyOption) {
    vector<string> input = {"-o", "nonempty"};
    EXPECT_TRUE(extractNonemptyOption(&input));
    // the '-o' has to go as well, libfuse rejects a '-o' without a value
    EXPECT_VECTOR_EQ({}, input);
}

TEST_F(ProgramOptionsUtilsTest, ExtractNonemptyOption_WithOtherDashOOptions) {
    vector<string> input = {"-o", "allow_other", "-o", "nonempty", "-o", "noatime"};
    EXPECT_TRUE(extractNonemptyOption(&input));
    EXPECT_VECTOR_EQ({"-o", "allow_other", "-o", "noatime"}, input);
}

TEST_F(ProgramOptionsUtilsTest, ExtractNonemptyOption_InACommaSeparatedList) {
    vector<string> input = {"-o", "allow_other,nonempty,noatime"};
    EXPECT_TRUE(extractNonemptyOption(&input));
    EXPECT_VECTOR_EQ({"-o", "allow_other,noatime"}, input);
}

TEST_F(ProgramOptionsUtilsTest, ExtractNonemptyOption_FirstInACommaSeparatedList) {
    vector<string> input = {"-o", "nonempty,allow_other"};
    EXPECT_TRUE(extractNonemptyOption(&input));
    EXPECT_VECTOR_EQ({"-o", "allow_other"}, input);
}

TEST_F(ProgramOptionsUtilsTest, ExtractNonemptyOption_LastInACommaSeparatedList) {
    vector<string> input = {"-o", "allow_other,nonempty"};
    EXPECT_TRUE(extractNonemptyOption(&input));
    EXPECT_VECTOR_EQ({"-o", "allow_other"}, input);
}

TEST_F(ProgramOptionsUtilsTest, ExtractNonemptyOption_OtherArgumentsAreKept) {
    vector<string> input = {"-f", "-o", "nonempty", "--logfile", "mylog"};
    EXPECT_TRUE(extractNonemptyOption(&input));
    EXPECT_VECTOR_EQ({"-f", "--logfile", "mylog"}, input);
}

TEST_F(ProgramOptionsUtilsTest, ExtractNonemptyOption_SeveralTimes) {
    vector<string> input = {"-o", "nonempty", "-o", "allow_other,nonempty"};
    EXPECT_TRUE(extractNonemptyOption(&input));
    EXPECT_VECTOR_EQ({"-o", "allow_other"}, input);
}

TEST_F(ProgramOptionsUtilsTest, ExtractNonemptyOption_WithoutDashO_IsNotTouched) {
    // an argument that happens to say 'nonempty' but isn't a mount option value
    vector<string> input = {"--logfile", "nonempty"};
    EXPECT_FALSE(extractNonemptyOption(&input));
    EXPECT_VECTOR_EQ({"--logfile", "nonempty"}, input);
}

TEST_F(ProgramOptionsUtilsTest, ExtractNonemptyOption_SimilarlyNamedOptionIsNotTouched) {
    vector<string> input = {"-o", "nonempty_something,somethingnonempty"};
    EXPECT_FALSE(extractNonemptyOption(&input));
    EXPECT_VECTOR_EQ({"-o", "nonempty_something,somethingnonempty"}, input);
}
