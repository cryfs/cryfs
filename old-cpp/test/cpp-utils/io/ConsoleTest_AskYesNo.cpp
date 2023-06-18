#include "ConsoleTest.h"

using std::string;

class ConsoleTest_AskYesNo: public ConsoleTest {
public:
    void EXPECT_TRUE_ON_INPUT(const string &input) {
        EXPECT_RESULT_ON_INPUT(true, input);
    }

    void EXPECT_FALSE_ON_INPUT(const string &input) {
        EXPECT_RESULT_ON_INPUT(false, input);
    }

    void EXPECT_RESULT_ON_INPUT(const bool expected, const string &input) {
        auto chosen = askYesNo("Are you sure blablub?");
        EXPECT_OUTPUT_LINES({"Are you sure blablub?"});
        EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
        sendInputLine(input);
        EXPECT_EQ(expected, chosen.get());
    }
};

TEST_F(ConsoleTest_AskYesNo, Input_Yes) {
    EXPECT_TRUE_ON_INPUT("Yes");
}

TEST_F(ConsoleTest_AskYesNo, Input_yes) {
    EXPECT_TRUE_ON_INPUT("yes");
}

TEST_F(ConsoleTest_AskYesNo, Input_Y) {
    EXPECT_TRUE_ON_INPUT("Y");
}

TEST_F(ConsoleTest_AskYesNo, Input_y) {
    EXPECT_TRUE_ON_INPUT("y");
}

TEST_F(ConsoleTest_AskYesNo, Input_No) {
    EXPECT_FALSE_ON_INPUT("No");
}

TEST_F(ConsoleTest_AskYesNo, Input_no) {
    EXPECT_FALSE_ON_INPUT("no");
}

TEST_F(ConsoleTest_AskYesNo, Input_N) {
    EXPECT_FALSE_ON_INPUT("N");
}

TEST_F(ConsoleTest_AskYesNo, Input_n) {
    EXPECT_FALSE_ON_INPUT("n");
}

TEST_F(ConsoleTest_AskYesNo, InputWithLeadingSpaces) {
    EXPECT_TRUE_ON_INPUT("  y");
}

TEST_F(ConsoleTest_AskYesNo, InputWithFollowingSpaces) {
    EXPECT_TRUE_ON_INPUT("y  ");
}

TEST_F(ConsoleTest_AskYesNo, InputWithLeadingAndFollowingSpaces) {
    EXPECT_TRUE_ON_INPUT("  y  ");
}

TEST_F(ConsoleTest_AskYesNo, InputEmptyLine) {
    auto chosen = askYesNo("My Question?");
    EXPECT_OUTPUT_LINES({"My Question?"});
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine(" "); // empty line with space
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("y");
    EXPECT_EQ(true, chosen.get());
}

TEST_F(ConsoleTest_AskYesNo, WrongInput) {
    auto chosen = askYesNo("My Question?");
    EXPECT_OUTPUT_LINES({"My Question?"});
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("0");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("1");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("bla");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("Y_andsomethingelse");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("y_andsomethingelse");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("N_andsomethingelse");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("n_andsomethingelse");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("Yes_andsomethingelse");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("yes_andsomethingelse");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("No_andsomethingelse");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("no_andsomethingelse");
    EXPECT_OUTPUT_LINE("Your choice [y/n]", ':', " ");
    sendInputLine("y");
    EXPECT_EQ(true, chosen.get());
}
