#pragma once
#ifndef MESSMER_CPPUTILS_IO_IOSTREAMCONSOLE_H
#define MESSMER_CPPUTILS_IO_IOSTREAMCONSOLE_H

#include "Console.h"

namespace cpputils {

    class IOStreamConsole final: public Console {
    public:
        IOStreamConsole();
        IOStreamConsole(std::ostream &output, std::istream &input);
        unsigned int ask(const std::string &question, const std::vector<std::string> &options) override;
        bool askYesNo(const std::string &question, bool defaultValue) override;
        void print(const std::string &output) override;
        std::string askPassword(const std::string &question) override;
    private:
        template<typename Return>
        Return _askForChoice(const std::string &question, std::function<boost::optional<Return> (const std::string&)> parse);
        static std::function<boost::optional<bool>(const std::string &input)> _parseYesNo();
        static std::function<boost::optional<unsigned int>(const std::string &input)> _parseUIntWithMinMax(unsigned int min, unsigned int max);
        static boost::optional<int> _parseInt(const std::string &str);

        std::ostream &_output;
        std::istream &_input;

        DISALLOW_COPY_AND_ASSIGN(IOStreamConsole);
    };

}
#endif
