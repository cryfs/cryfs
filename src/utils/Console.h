#pragma once
#ifndef CRYFS_CONSOLE_H
#define CRYFS_CONSOLE_H

#include <string>
#include <vector>
#include <iostream>

//TODO Add test cases

class Console {
public:
    Console();
    Console(std::ostream &output, std::istream &input);
    unsigned int ask(const std::string &question, const std::vector<std::string> &options);
private:
    std::ostream &_output;
    std::istream &_input;
};


#endif
