#ifndef CRYFS_IOEXCEPTION_H
#define CRYFS_IOEXCEPTION_H

#include <stdexcept>

namespace fspp {

    class IOException : public std::exception {
    public:
        IOException(const std::string &message) : _message(message) { }

        const char *what() const throw() override {
            return _message.c_str();
        }

    private:
        std::string _message;
    };

}

#endif
