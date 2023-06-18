#include "ConsoleTest.h"

using std::stringstream;
using std::string;
using std::istream;
using std::ostream;

class ConsoleTest_Ask: public ConsoleTest {};

TEST_F(ConsoleTest_Ask, CrashesWithoutOptions) {
  EXPECT_THROW(
    (ask("My Question?", {}).get()),
    std::invalid_argument
  );
}

TEST_F(ConsoleTest_Ask, OneOption) {
  auto chosen = ask("My Question?", {"First Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-1]", ':', " ");
  sendInputLine("1");
  EXPECT_EQ(0u, chosen.get());
}

TEST_F(ConsoleTest_Ask, TwoOptions_ChooseFirst) {
  auto chosen = ask("My Question?", {"First Option", "Second Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option",
    " [2] Second Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("1");
  EXPECT_EQ(0u, chosen.get());
}

TEST_F(ConsoleTest_Ask, TwoOptions_ChooseSecond) {
  auto chosen = ask("My Question?", {"First Option", "Second Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option",
    " [2] Second Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("2");
  EXPECT_EQ(1u, chosen.get());
}

TEST_F(ConsoleTest_Ask, ThreeOptions_ChooseFirst) {
  auto chosen = ask("My Other Question?", {"1st Option", "2nd Option", "3rd Option"});
  EXPECT_OUTPUT_LINES({
    "My Other Question?",
    " [1] 1st Option",
    " [2] 2nd Option",
    " [3] 3rd Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-3]", ':', " ");
  sendInputLine("1");
  EXPECT_EQ(0u, chosen.get());
}

TEST_F(ConsoleTest_Ask, ThreeOptions_ChooseSecond) {
  auto chosen = ask("My Question?", {"1st Option", "2nd Option", "3rd Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] 1st Option",
    " [2] 2nd Option",
    " [3] 3rd Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-3]", ':', " ");
  sendInputLine("2");
  EXPECT_EQ(1u, chosen.get());
}

TEST_F(ConsoleTest_Ask, ThreeOptions_ChooseThird) {
  auto chosen = ask("My Question?", {"1st Option", "2nd Option", "3rd Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] 1st Option",
    " [2] 2nd Option",
    " [3] 3rd Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-3]", ':', " ");
  sendInputLine("3");
  EXPECT_EQ(2u, chosen.get());
}

TEST_F(ConsoleTest_Ask, InputWithLeadingSpaces) {
  auto chosen = ask("My Question?", {"First Option", "Second Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option",
    " [2] Second Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("  2");
  EXPECT_EQ(1u, chosen.get());
}

TEST_F(ConsoleTest_Ask, InputWithFollowingSpaces) {
  auto chosen = ask("My Question?", {"First Option", "Second Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option",
    " [2] Second Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("2  ");
  EXPECT_EQ(1u, chosen.get());
}

TEST_F(ConsoleTest_Ask, InputWithLeadingAndFollowingSpaces) {
  auto chosen = ask("My Question?", {"First Option", "Second Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option",
    " [2] Second Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("  2  ");
  EXPECT_EQ(1u, chosen.get());
}

TEST_F(ConsoleTest_Ask, InputEmptyLine) {
  auto chosen = ask("My Question?", {"First Option", "Second Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option",
    " [2] Second Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("");
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine(" "); // empty line with space
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("2");
  EXPECT_EQ(1u, chosen.get());
}

TEST_F(ConsoleTest_Ask, InputWrongNumbers) {
  auto chosen = ask("My Question?", {"1st Option", "2nd Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] 1st Option",
    " [2] 2nd Option",
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("0");
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("-1");
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("3");
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("1.5");
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("1,5");
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("2");
  EXPECT_EQ(1u, chosen.get());
}

TEST_F(ConsoleTest_Ask, InputNonNumbers) {
  auto chosen = ask("My Question?", {"1st Option", "2nd Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] 1st Option",
    " [2] 2nd Option",
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("abc");
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("3a"); // Wrong number and string attached
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("1a"); // Right number but string attached
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("a3"); // Wrong number and string attached
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("a1"); // Right number but string attached
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("2");
  EXPECT_EQ(1u, chosen.get());
}
