#include <google/gtest/gtest.h>

#include "../../src/utils/Console.h"

#include <future>
#include <thread>
#include <messmer/cpp-utils/pipestream.h>

using std::stringstream;
using std::string;
using std::vector;
using std::istream;
using std::ostream;
using std::future;
using std::initializer_list;

class ConsoleThread {
public:
    ConsoleThread(ostream &ostr, istream &istr): _console(ostr, istr) {}
    future<unsigned int> ask(const string &question, const vector<string> &options) {
        return std::async(std::launch::async, [this, question, options]() {
           return _console.ask(question, options);
        });
    }
    void print(const string &output) {
      _console.print(output);
    }
private:
    IOStreamConsole _console;
};

class ConsoleTest: public ::testing::Test {
public:
    ConsoleTest(): _inputStr(), _outputStr(), _input(&_inputStr), _output(&_outputStr), _console(_output, _input) {}

    void EXPECT_OUTPUT_LINES(initializer_list<string> lines) {
        for (const string &line : lines) {
            EXPECT_OUTPUT_LINE(line);
        }
    }

    void EXPECT_OUTPUT_LINE(const string &expected, char delimiter = '\n', const string &expected_after_delimiter = "") {
        string actual;
        std::getline(_output, actual, delimiter);
        EXPECT_EQ(expected, actual);
        for (char expected_char : expected_after_delimiter) {
            char actual_char;
            _output.get(actual_char);
            EXPECT_EQ(expected_char, actual_char);
        }
    }

    void sendInputLine(const string &line) {
        _input << line << "\n" << std::flush;
    }

    future<unsigned int> ask(const string &question, const vector<string> &options) {
        return _console.ask(question, options);
    }

    void print(const string &output) {
      _console.print(output);
    }

private:
    cpputils::pipestream _inputStr;
    cpputils::pipestream _outputStr;
    std::iostream _input;
    std::iostream _output;
    ConsoleThread _console;
};

TEST_F(ConsoleTest, AskCrashesWithoutOptions) {
  EXPECT_THROW(
    (ask("My Question?", {}).get()),
    std::invalid_argument
  );
}

TEST_F(ConsoleTest, AskOneOption) {
  auto chosen = ask("My Question?", {"First Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-1]", ':', " ");
  sendInputLine("1");
  EXPECT_EQ(0, chosen.get());
}

TEST_F(ConsoleTest, AskTwoOptions_ChooseFirst) {
  auto chosen = ask("My Question?", {"First Option", "Second Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option",
    " [2] Second Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("1");
  EXPECT_EQ(0, chosen.get());
}

TEST_F(ConsoleTest, AskTwoOptions_ChooseSecond) {
  auto chosen = ask("My Question?", {"First Option", "Second Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option",
    " [2] Second Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("2");
  EXPECT_EQ(1, chosen.get());
}

TEST_F(ConsoleTest, AskThreeOptions_ChooseFirst) {
  auto chosen = ask("My Other Question?", {"1st Option", "2nd Option", "3rd Option"});
  EXPECT_OUTPUT_LINES({
    "My Other Question?",
    " [1] 1st Option",
    " [2] 2nd Option",
    " [3] 3rd Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-3]", ':', " ");
  sendInputLine("1");
  EXPECT_EQ(0, chosen.get());
}

TEST_F(ConsoleTest, AskThreeOptions_ChooseSecond) {
  auto chosen = ask("My Question?", {"1st Option", "2nd Option", "3rd Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] 1st Option",
    " [2] 2nd Option",
    " [3] 3rd Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-3]", ':', " ");
  sendInputLine("2");
  EXPECT_EQ(1, chosen.get());
}

TEST_F(ConsoleTest, AskThreeOptions_ChooseThird) {
  auto chosen = ask("My Question?", {"1st Option", "2nd Option", "3rd Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] 1st Option",
    " [2] 2nd Option",
    " [3] 3rd Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-3]", ':', " ");
  sendInputLine("3");
  EXPECT_EQ(2, chosen.get());
}

TEST_F(ConsoleTest, InputWithLeadingSpaces) {
  auto chosen = ask("My Question?", {"First Option", "Second Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option",
    " [2] Second Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("  2");
  EXPECT_EQ(1, chosen.get());
}

TEST_F(ConsoleTest, InputWithFollowingSpaces) {
  auto chosen = ask("My Question?", {"First Option", "Second Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option",
    " [2] Second Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("2  ");
  EXPECT_EQ(1, chosen.get());
}

TEST_F(ConsoleTest, InputWithLeadingAndFollowingSpaces) {
  auto chosen = ask("My Question?", {"First Option", "Second Option"});
  EXPECT_OUTPUT_LINES({
    "My Question?",
    " [1] First Option",
    " [2] Second Option"
  });
  EXPECT_OUTPUT_LINE("Your choice [1-2]", ':', " ");
  sendInputLine("  2  ");
  EXPECT_EQ(1, chosen.get());
}

TEST_F(ConsoleTest, InputEmptyLine) {
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
  EXPECT_EQ(1, chosen.get());
}

TEST_F(ConsoleTest, InputWrongNumbers) {
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
  EXPECT_EQ(1, chosen.get());
}

TEST_F(ConsoleTest, InputNonNumbers) {
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
  EXPECT_EQ(1, chosen.get());
}

TEST_F(ConsoleTest, TestPrint) {
  print("Bla Blub");
  EXPECT_OUTPUT_LINE("Bla Blu", 'b'); // 'b' is the delimiter for reading
}