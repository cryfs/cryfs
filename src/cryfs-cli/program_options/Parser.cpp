#include "Parser.h"
#include "utils.h"
#include <iostream>
#include <boost/optional.hpp>
#include <cryfs/config/CryConfigConsole.h>
#include <cryfs/CryfsException.h>
#include <cryfs-cli/Environment.h>

namespace po = boost::program_options;
namespace bf = boost::filesystem;
using namespace cryfs::program_options;
using cryfs::CryConfigConsole;
using cryfs::CryfsException;
using std::pair;
using std::vector;
using std::cerr;
using std::cout;
using std::endl;
using std::string;
using boost::optional;
using boost::none;

Parser::Parser(int argc, const char *argv[])
        :_options(_argsToVector(argc, argv)) {
}

vector<string> Parser::_argsToVector(int argc, const char *argv[]) {
    vector<string> result;
    for(int i = 0; i < argc; ++i) {
        result.push_back(argv[i]);
    }
    return result;
}

ProgramOptions Parser::parse(const vector<string> &supportedCiphers) const {
    pair<vector<string>, vector<string>> options = splitAtDoubleDash(_options);
    po::variables_map vm = _parseOptionsOrShowHelp(options.first, supportedCiphers);

    if (!vm.count("base-dir")) {
        _showHelpAndExit("Please specify a base directory.", ErrorCode::InvalidArguments);
    }
    if (!vm.count("mount-dir")) {
        _showHelpAndExit("Please specify a mount directory.", ErrorCode::InvalidArguments);
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
    bool allowFilesystemUpgrade = vm.count("allow-filesystem-upgrade");
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
    optional<uint32_t> blocksizeBytes = none;
    if (vm.count("blocksize")) {
        blocksizeBytes = vm["blocksize"].as<uint32_t>();
    }

    return ProgramOptions(baseDir, mountDir, configfile, foreground, allowFilesystemUpgrade, unmountAfterIdleMinutes, logfile, cipher, blocksizeBytes, options.second);
}

void Parser::_checkValidCipher(const string &cipher, const vector<string> &supportedCiphers) {
    if (std::find(supportedCiphers.begin(), supportedCiphers.end(), cipher) == supportedCiphers.end()) {
        throw CryfsException("Invalid cipher: " + cipher, ErrorCode::InvalidArguments);
    }
}

po::variables_map Parser::_parseOptionsOrShowHelp(const vector<string> &options, const vector<string> &supportedCiphers) {
    try {
      return _parseOptions(options, supportedCiphers);
    } catch (const CryfsException& e) {
        // If CryfsException is thrown, we already know what's wrong.
        // Show usage information and pass through the exception, don't catch it.
        if (e.errorCode() != ErrorCode::Success) {
          _showHelp();
        }
        throw;
    } catch(const std::exception &e) {
        std::cerr << e.what() << std::endl;
        _showHelpAndExit("Invalid arguments", ErrorCode::InvalidArguments);
    }
}

po::variables_map Parser::_parseOptions(const vector<string> &options, const vector<string> &supportedCiphers) {
    po::options_description desc;
    po::positional_options_description positional_desc;
    _addAllowedOptions(&desc);
    _addPositionalOptionForBaseDir(&desc, &positional_desc);

    po::variables_map vm;
    vector<const char*> _options = _to_const_char_vector(options);
    po::store(po::command_line_parser(_options.size(), _options.data())
                      .options(desc).positional(positional_desc).run(), vm);
    if (vm.count("help")) {
        _showHelpAndExit("", ErrorCode::Success);
    }
    if (vm.count("show-ciphers")) {
        _showCiphersAndExit(supportedCiphers);
    }
    if (vm.count("version")) {
        _showVersionAndExit();
    }
    po::notify(vm);

    return vm;
}

vector<const char*> Parser::_to_const_char_vector(const vector<string> &options) {
    vector<const char*> result;
    result.reserve(options.size());
    for (const string &option : options) {
        result.push_back(option.c_str());
    }
    return result;
}

void Parser::_addAllowedOptions(po::options_description *desc) {
    po::options_description options("Allowed options");
    string cipher_description = "Cipher to use for encryption. See possible values by calling cryfs with --show-ciphers. Default: ";
    cipher_description += CryConfigConsole::DEFAULT_CIPHER;
    string blocksize_description = "The block size used when storing ciphertext blocks (in bytes). Default: ";
    blocksize_description += std::to_string(CryConfigConsole::DEFAULT_BLOCKSIZE_BYTES);
    options.add_options()
            ("help,h", "show help message")
            ("config,c", po::value<string>(), "Configuration file")
            ("foreground,f", "Run CryFS in foreground.")
            ("cipher", po::value<string>(), cipher_description.c_str())
            ("blocksize", po::value<uint32_t>(), blocksize_description.c_str())
            ("allow-filesystem-upgrade", "Allow upgrading the file system if it was created with an old CryFS version. After the upgrade, older CryFS versions might not be able to use the file system anymore.")
            ("show-ciphers", "Show list of supported ciphers.")
            ("unmount-idle", po::value<double>(), "Automatically unmount after specified number of idle minutes.")
            ("logfile", po::value<string>(), "Specify the file to write log messages to. If this is not specified, log messages will go to stdout, or syslog if CryFS is running in the background.")
            ("version", "Show CryFS version number")
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
    throw CryfsException("", ErrorCode::Success);
}

void Parser::_showHelp() {
  cerr << "Usage: cryfs [options] baseDir mountPoint [-- [FUSE Mount Options]]\n";
  po::options_description desc;
  _addAllowedOptions(&desc);
  cerr << desc << endl;
  cerr << "Environment variables:\n"
       << "  " << Environment::FRONTEND_KEY << "=" << Environment::FRONTEND_NONINTERACTIVE << "\n"
       << "\tWork better together with tools.\n"
       << "\tWith this option set, CryFS won't ask anything, but use default values\n"
       << "\tfor options you didn't specify on command line. Furthermore, it won't\n"
       << "\task you to enter a new password a second time (password confirmation).\n"
       << "  " << Environment::NOUPDATECHECK_KEY << "=true\n"
       << "\tBy default, CryFS connects to the internet to check for known\n"
       << "\tsecurity vulnerabilities and new versions. This option disables this.\n"
       << endl;
}

[[noreturn]] void Parser::_showHelpAndExit(const std::string& message, ErrorCode errorCode) {
    _showHelp();
    throw CryfsException(message, errorCode);
}

[[noreturn]] void Parser::_showVersionAndExit() {
  // no need to show version because it was already shown in the CryFS header before parsing program options
    throw CryfsException("", ErrorCode::Success);
}
