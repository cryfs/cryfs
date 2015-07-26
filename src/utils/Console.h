#pragma once
#ifndef CRYFS_CONSOLE_H
#define CRYFS_CONSOLE_H

#include <string>
#include <vector>
#include <iostream>

class Console {
public:
    virtual unsigned int ask(const std::string &question, const std::vector<std::string> &options) = 0;
    virtual void print(const std::string &output) = 0;
};

class IOStreamConsole: public Console {
public:
    IOStreamConsole();
    IOStreamConsole(std::ostream &output, std::istream &input);
    unsigned int ask(const std::string &question, const std::vector<std::string> &options) override;

    //TODO Test print()
    void print(const std::string &output) override;
private:
    std::ostream &_output;
    std::istream &_input;
};


#endif
