#include "Parser.h"
#include "utils.h"
#include <iostream>
#include <boost/optional.hpp>

namespace po = boost::program_options;
namespace bf = boost::filesystem;
using namespace cryfs::program_options;
using std::pair;
using std::vector;
using std::cerr;
using std::endl;
using std::string;
using boost::optional;
using boost::none;

Parser::Parser(int argc, char *argv[])
        :_options(_argsToVector(argc, argv)) {
}

vector<char*> Parser::_argsToVector(int argc, char *argv[]) {
    vector<char*> result;
    for(int i = 0; i < argc; ++i) {
        result.push_back(argv[i]);
    }
    return result;
}

ProgramOptions Parser::parse(const vector<string> &supportedCiphers) const {
    pair<vector<char*>, vector<char*>> options = splitAtDoubleDash(_options);
    po::variables_map vm = _parseOptionsOrShowHelp(options.first, supportedCiphers);

    if (!vm.count("base-dir")) {
        std::cerr << "Please specify a base directory.\n";
        _showHelpAndExit();
    }
    if (!vm.count("mount-dir")) {
        std::cerr << "Please specify a mount directory.\n";
        _showHelpAndExit();
    }
    bf::path baseDir = bf::absolute(vm["base-dir"].as<string>());
    bf::path mountDir = bf::absolute(vm["mount-dir"].as<string>());
    optional<bf::path> configfile = none;
    if (vm.count("config")) {
        configfile = bf::absolute(vm["config"].as<string>());
    }
    bool foreground = vm.count("foreground");
    if (foreground) {
        options.second.push_back(const_cast<char*>("-f"));
    }
    optional<double> unmountAfterIdleMinutes = none;
    if (vm.count("unmount-idle")) {
        unmountAfterIdleMinutes = vm["unmount-idle"].as<double>();
    }
    optional<bf::path> logfile = none;
    if (vm.count("logfile")) {
        logfile = bf::absolute(vm["logfile"].as<string>());
    }
    optional<string> cipher = none;
    if (vm.count("cipher")) {
        cipher = vm["cipher"].as<string>();
        _checkValidCipher(*cipher, supportedCiphers);
    }

    return ProgramOptions(baseDir, mountDir, configfile, foreground, unmountAfterIdleMinutes, logfile, cipher, options.second);
}

void Parser::_checkValidCipher(const string &cipher, const vector<string> &supportedCiphers) {
    if (std::find(supportedCiphers.begin(), supportedCiphers.end(), cipher) == supportedCiphers.end()) {
        std::cerr << "Invalid cipher: " << cipher << std::endl;
        exit(1);
    }
}

po::variables_map Parser::_parseOptionsOrShowHelp(const vector<char*> options, const vector<string> &supportedCiphers) {
    try {
        return _parseOptions(options, supportedCiphers);
    } catch(const std::exception &e) {
        std::cerr << e.what() << std::endl;
        _showHelpAndExit();
    }
}

po::variables_map Parser::_parseOptions(const vector<char*> options, const vector<string> &supportedCiphers) {
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
    if (vm.count("show-ciphers")) {
        _showCiphersAndExit(supportedCiphers);
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
            ("cipher", po::value<string>(), "Cipher to use for encryption. See possible values by calling cryfs with --show-ciphers")
            ("show-ciphers", "Show list of supported ciphers.")
            ("unmount-idle", po::value<double>(), "Automatically unmount after specified number of idle minutes.")
            ("logfile", po::value<string>(), "Specify the file to write log messages to. If this is not specified, log messages will go to stdout, or syslog if CryFS is running in the background.")
            ;
    desc->add(options);
}

void Parser::_addPositionalOptionForBaseDir(po::options_description *desc, po::positional_options_description *positional) {
    positional->add("base-dir", 1);
    positional->add("mount-dir", 1);
    po::options_description hidden("Hidden options");
    hidden.add_options()
            ("base-dir", po::value<string>(), "Base directory")
            ("mount-dir", po::value<string>(), "Mount directory")
            ;
    desc->add(hidden);
}

[[noreturn]] void Parser::_showCiphersAndExit(const vector<string> &supportedCiphers) {
    for (const auto &cipher : supportedCiphers) {
        std::cerr << cipher << "\n";
    }
    exit(0);
}

[[noreturn]] void Parser::_showHelpAndExit() {
    cerr << "Usage: cryfs [options] baseDir mountPoint [-- [FUSE Mount Options]]\n";
    po::options_description desc;
    _addAllowedOptions(&desc);
    cerr << desc << endl;
    exit(1);
}
