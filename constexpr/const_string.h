#ifndef MESSMER_CPPUTILS_CONSTEXPR_CONST_STRING_H
#define MESSMER_CPPUTILS_CONSTEXPR_CONST_STRING_H

#include <cstring>
#include <string>
#include <iostream>
#include "impl/digit_parser.h"

namespace cpputils {
    class const_string {
    public:
        constexpr const_string(const char *str) : const_string(str, strlen(str)) { }

        constexpr unsigned int size() const {
            return _size;
        }

        constexpr char operator[](unsigned int index) const {
            return (index < size()) ? _str[index] : throw std::logic_error("Index out of bounds");
        }

        constexpr const_string dropPrefix(unsigned int prefixSize) const {
            return substr(prefixSize, _size - prefixSize);
        }

        constexpr const_string dropSuffix(unsigned int suffixSize) const {
            return substr(0, _size - suffixSize);
        }

        constexpr const_string substr(unsigned int start, unsigned int count) const {
            return (start + count <= size()) ? const_string(_str + start, count)
                                             : throw std::logic_error("Substring out of bounds");
        }

        constexpr unsigned int sizeOfUIntPrefix() const {
            return _hasUIntPrefix() ? (1 + dropPrefix(1).sizeOfUIntPrefix()) : 0;
        }

        constexpr unsigned int parseUIntPrefix() const {
            return _hasUIntPrefix() ? _parseUIntBackwards(_str + sizeOfUIntPrefix() - 1, sizeOfUIntPrefix())
                                    : throw std::logic_error("Not a valid number");
        }

        constexpr const_string dropUIntPrefix() const {
            return _hasUIntPrefix() ? dropPrefix(1).dropUIntPrefix() : *this;
        }

        constexpr bool operator==(const const_string &rhs) const {
            return _size == rhs._size && 0 == memcmp(_str, rhs._str, _size);
        }

        constexpr bool operator!=(const const_string &rhs) const {
            return !operator==(rhs);
        }

        std::string toStdString() const {
            return std::string(_str, _size);
        }

    private:
        constexpr const_string(const char *str, unsigned int size) : _str(str), _size(size) { }

        static constexpr unsigned int _parseUIntBackwards(const char *input, unsigned int numDigits) {
            return (numDigits == 0) ? 0 : (digit_parser::parseDigit(*input) +
                                           10 * _parseUIntBackwards(input - 1, numDigits - 1));
        }

        constexpr bool _hasUIntPrefix() const {
            return size() > 0 && digit_parser::isDigit(_str[0]);
        }

        const char *_str;
        unsigned int _size;
    };

    std::ostream &operator<<(std::ostream &stream, const const_string &str) {
        stream << str.toStdString();
        return stream;
    }
}

#endif
