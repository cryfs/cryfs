#include "Parser.h"
#include "utils.h"
#include <iostream>
#include <boost/optional.hpp>

namespace po = boost::program_options;
using namespace cryfs::program_options;
using std::pair;
using std::vector;
using std::cerr;
using std::endl;
using std::string;
using boost::optional;
using boost::none;

Parser::Parser(int argc, char *argv[]) :_options(_argsToVector(argc, argv)) {}

vector<char*> Parser::_argsToVector(int argc, char *argv[]) {
    vector<char*> result;
    for(int i = 0; i < argc; ++i) {
        result.push_back(argv[i]);
    }
    return result;
}

ProgramOptions Parser::parse() const {
    pair<vector<char*>, vector<char*>> options = splitAtDoubleDash(_options);
    po::variables_map vm = _parseOptionsOrShowHelp(options.first);

    string baseDir = vm["base-dir"].as<string>();
    string mountDir = vm["mount-dir"].as<string>();
    optional<string> configfile = none;
    if (vm.count("config")) {
        configfile = vm["config"].as<string>();
    }
    bool foreground = vm.count("foreground");
    optional<string> logfile = none;
    if (vm.count("logfile")) {
        logfile = vm["logfile"].as<string>();
    }

    return ProgramOptions(baseDir, mountDir, configfile, foreground, logfile, options.second);
}

po::variables_map Parser::_parseOptionsOrShowHelp(const vector<char*> options) {
    try {
        return _parseOptions(options);
    } catch(const std::exception &e) {
        _showHelpAndExit();
    }
}

po::variables_map Parser::_parseOptions(const vector<char*> options) {
    po::options_description desc;
    po::positional_options_description positional_desc;
    _addAllowedOptions(&desc);
    _addPositionalOptionForBaseDir(&desc, &positional_desc);

    po::variables_map vm;
    po::store(po::command_line_parser(options.size(), options.data())
                      .options(desc).positional(positional_desc).run(), vm);
    if (vm.count("help")) {
        _showHelpAndExit();
    }
    po::notify(vm);

    return vm;
}

void Parser::_addAllowedOptions(po::options_description *desc) {
    po::options_description options("Allowed options");
    options.add_options()
            ("help,h", "show help message")
            ("config,c", po::value<string>(), "Configuration file")
            ("foreground,f", "Run CryFS in foreground.")
            ("logfile", po::value<string>(), "Specify the file to write log messages to. If this is not specified, log messages will go to stdout, or syslog if CryFS is running in the background.")
            ;
    desc->add(options);
}

void Parser::_addPositionalOptionForBaseDir(po::options_description *desc, po::positional_options_description *positional) {
    positional->add("base-dir", 1);
    positional->add("mount-dir", 1);
    po::options_description hidden("Hidden options");
    hidden.add_options()
            ("base-dir", po::value<string>()->required(), "Base directory")
            ("mount-dir", po::value<string>()->required(), "Mount directory")
            ;
    desc->add(hidden);
}

[[noreturn]] void Parser::_showHelpAndExit() {
    cerr << "Usage: cryfs [options] rootDir mountPoint [-- [FUSE Mount Options]]\n";
    po::options_description desc;
    _addAllowedOptions(&desc);
    cerr << desc << endl;
    exit(1);
}
