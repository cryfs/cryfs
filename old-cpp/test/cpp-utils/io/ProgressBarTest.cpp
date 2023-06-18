#include <cpp-utils/io/ProgressBar.h>
#include <gmock/gmock.h>

using cpputils::ProgressBar;
using std::make_shared;

class MockConsole final: public cpputils::Console {
public:
    void EXPECT_OUTPUT(const char* expected) {
        EXPECT_EQ(expected, _output);
        _output = "";
    }

    void print(const std::string& text) override {
        _output += text;
    }

    unsigned int ask(const std::string&, const std::vector<std::string>&) override {
        EXPECT_TRUE(false);
        return 0;
    }

    bool askYesNo(const std::string&, bool) override {
        EXPECT_TRUE(false);
        return false;
    }

    std::string askPassword(const std::string&) override {
        EXPECT_TRUE(false);
        return "";
    }

private:
    std::string _output;
};

TEST(ProgressBarTest, testProgressBar) {
    auto console = make_shared<MockConsole>();

    ProgressBar bar(console, "Preamble", 2000);
    console->EXPECT_OUTPUT("\n\rPreamble 0%");

    // when updating to 0, doesn't reprint
    bar.update(0);
    console->EXPECT_OUTPUT("");

    // update to half
    bar.update(1000);
    console->EXPECT_OUTPUT("\rPreamble 50%");

    // when updating to same value, doesn't reprint
    bar.update(1000);
    console->EXPECT_OUTPUT("");

    // when updating to value with same percentage, doesn't reprint
    bar.update(1001);
    console->EXPECT_OUTPUT("");

    // update to 0
    bar.update(0);
    console->EXPECT_OUTPUT("\rPreamble 0%");

    // update to full
    bar.update(2000);
    console->EXPECT_OUTPUT("\rPreamble 100%");
}
