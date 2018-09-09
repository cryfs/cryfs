#include "ConsoleTest.h"

using std::stringstream;
using std::string;
using std::istream;
using std::ostream;

class ConsoleTest_AskPassword: public ConsoleTest {};

TEST_F(ConsoleTest_AskPassword, InputSomePassword) {
  auto chosen = askPassword("Please enter my password:");
  EXPECT_OUTPUT_LINE("Please enter my password", ':');
  sendInputLine("this is the password");
  EXPECT_EQ("this is the password", chosen.get());
}

TEST_F(ConsoleTest_AskPassword, InputEmptyPassword) {
  auto chosen = askPassword("Please enter my password:");
  EXPECT_OUTPUT_LINE("Please enter my password", ':');
  sendInputLine("");
  EXPECT_EQ("", chosen.get());
}
