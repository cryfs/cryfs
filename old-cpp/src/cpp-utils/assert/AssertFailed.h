#pragma once
#ifndef MESSMER_CPPUTILS_ASSERT_ASSERTFAILED_H
#define MESSMER_CPPUTILS_ASSERT_ASSERTFAILED_H

#include <stdexcept>
#include <string>
#include "../macros.h"

namespace cpputils {

    class AssertFailed final: public std::exception {
    public:
        explicit AssertFailed(std::string message) : _message(std::move(message)) { }

        const char *what() const throw() override {
            return _message.c_str();
        }

    private:
        std::string _message;
    };

}

#endif
