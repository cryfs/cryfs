#include "testutils/ProgramOptionsTestBase.h"
#include "../../src/program_options/utils.h"

using namespace cryfs::program_options;
using std::pair;
using std::vector;
using std::string;

class ProgramOptionsUtilsTest: public ProgramOptionsTestBase {};

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_ZeroOptions) {
    vector<char*> input = options({"./executableName"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneShortOption) {
    vector<char*> input = options({"./executableName", "-j"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "-j"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneLongOption) {
    vector<char*> input = options({"./executableName", "--myoption"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "--myoption"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OnePositionalOption) {
    vector<char*> input = options({"./executableName", "mypositionaloption"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneShortOption_DoubleDash) {
    vector<char*> input = options({"./executableName", "-j", "--"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "-j"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneLongOption_DoubleDash) {
    vector<char*> input = options({"./executableName", "--myoption", "--"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "--myoption"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OnePositionalOption_DoubleDash) {
    vector<char*> input = options({"./executableName", "mypositionaloption", "--"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_DoubleDash_OneShortOption) {
    vector<char*> input = options({"./executableName", "--", "-a"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName", "-a"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_DoubleDash_OneLongOption) {
    vector<char*> input = options({"./executableName", "--", "--myoption"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName", "--myoption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_DoubleDash_OnePositionalOption) {
    vector<char*> input = options({"./executableName", "--", "mypositionaloption"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneShortOption_DoubleDash_OneShortOption) {
    vector<char*> input = options({"./executableName", "-j", "--", "-a"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "-j"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName", "-a"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OneLongOption_DoubleDash_OneLongOption) {
    vector<char*> input = options({"./executableName", "--myoption", "--", "--myotheroption"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "--myoption"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName", "--myotheroption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_OnePositionalOption_DoubleDash_OnePositionalOption) {
    vector<char*> input = options({"./executableName", "mypositionaloption", "--", "otherpositionaloption"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName", "otherpositionaloption"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_MoreOptions) {
    vector<char*> input = options({"./executableName", "mypositionaloption", "myotherpositionaloption", "-j", "--alpha", "--", "filename", "--beta", "-j3"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "mypositionaloption", "myotherpositionaloption", "-j", "--alpha"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName", "filename", "--beta", "-j3"}, result.second);
}

TEST_F(ProgramOptionsUtilsTest, SplitAtDoubleDash_RealisticCryfsOptions) {
    vector<char*> input = options({"./executableName", "rootDir", "mountDir", "--", "-f"});
    pair<vector<char*>,vector<char*>> result = splitAtDoubleDash(input);
    EXPECT_VECTOR_EQ({"./executableName", "rootDir", "mountDir"}, result.first);
    EXPECT_VECTOR_EQ({"./executableName", "-f"}, result.second);
}
