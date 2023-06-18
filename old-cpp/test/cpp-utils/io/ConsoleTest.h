#pragma once
#ifndef MESSMER_CPPUTILS_TEST_IO_CONSOLETEST_H
#define MESSMER_CPPUTILS_TEST_IO_CONSOLETEST_H

#include <gtest/gtest.h>

#include "cpp-utils/io/IOStreamConsole.h"

#include <future>
#include <thread>
#include "cpp-utils/io/pipestream.h"

class ConsoleThread {
public:
    ConsoleThread(std::ostream &ostr, std::istream &istr): _console(ostr, istr) {}
    std::future<unsigned int> ask(const std::string &question, const std::vector<std::string> &options) {
        return std::async(std::launch::async, [this, question, options]() {
            return _console.ask(question, options);
        });
    }
    std::future<bool> askYesNo(const std::string &question) {
        return std::async(std::launch::async, [this, question]() {
            return _console.askYesNo(question, true);
        });
    }
    std::future<std::string> askPassword(const std::string &question) {
        return std::async(std::launch::async, [this, question]() {
            return _console.askPassword(question);
        });
    }
    void print(const std::string &output) {
        _console.print(output);
    }
private:
    cpputils::IOStreamConsole _console;
};

class ConsoleTest: public ::testing::Test {
public:
    ConsoleTest(): _inputStr(), _outputStr(), _input(&_inputStr), _output(&_outputStr), _console(_output, _input) {}

    void EXPECT_OUTPUT_LINES(std::initializer_list<std::string> lines) {
        for (const std::string &line : lines) {
            EXPECT_OUTPUT_LINE(line);
        }
    }

    void EXPECT_OUTPUT_LINE(const std::string &expected, char delimiter = '\n', const std::string &expected_after_delimiter = "") {
        std::string actual;
        std::getline(_output, actual, delimiter);
        EXPECT_EQ(expected, actual);
        for (char expected_char : expected_after_delimiter) {
            char actual_char = 0;
            _output.get(actual_char);
            EXPECT_EQ(expected_char, actual_char);
        }
    }

    void sendInputLine(const std::string &line) {
        _input << line << "\n" << std::flush;
    }

    std::future<unsigned int> ask(const std::string &question, const std::vector<std::string> &options) {
        return _console.ask(question, options);
    }

    std::future<bool> askYesNo(const std::string &question) {
        return _console.askYesNo(question);
    }

    std::future<std::string> askPassword(const std::string &question) {
        return _console.askPassword(question);
    }

    void print(const std::string &output) {
        _console.print(output);
    }

private:
    cpputils::pipestream _inputStr;
    cpputils::pipestream _outputStr;
    std::iostream _input;
    std::iostream _output;
    ConsoleThread _console;
};

#endif
