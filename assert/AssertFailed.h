#pragma once
#ifndef MESSMER_CPP_UTILS_ASSERT_ASSERTFAILED_H
#define MESSMER_CPP_UTILS_ASSERT_ASSERTFAILED_H

#include <stdexcept>
#include <string>

namespace cpputils {

    class AssertFailed : public std::exception {
    public:
        AssertFailed(const std::string &message) : _message(message) { }

        const char *what() const throw() override {
            return _message.c_str();
        }

    private:
        std::string _message;
    };

}

#endif
