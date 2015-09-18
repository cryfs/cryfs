#ifndef MESSMER_CPPUTILS_CONSTEXPR_DIGIT_PARSER_H
#define MESSMER_CPPUTILS_CONSTEXPR_DIGIT_PARSER_H

#include <stdexcept>

namespace cpputils {
    class digit_parser {
    public:
        static constexpr bool isDigit(char digit) {
            return digit == '0' || digit == '1' || digit == '2' || digit == '3' || digit == '4' || digit == '5' ||
                   digit == '6' || digit == '7' || digit == '8' || digit == '9';
        }

        static constexpr unsigned char parseDigit(char digit) {
            return (digit == '0') ? 0 :
                   (digit == '1') ? 1 :
                   (digit == '2') ? 2 :
                   (digit == '3') ? 3 :
                   (digit == '4') ? 4 :
                   (digit == '5') ? 5 :
                   (digit == '6') ? 6 :
                   (digit == '7') ? 7 :
                   (digit == '8') ? 8 :
                   (digit == '9') ? 9 :
                   throw std::logic_error("Not a valid digit");
        }
    };
}

#endif
