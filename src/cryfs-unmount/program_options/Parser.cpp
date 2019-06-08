#include "Parser.h"
#include <iostream>
#include <boost/optional.hpp>
#include <cryfs/config/CryConfigConsole.h>
#include <cryfs/CryfsException.h>
#include <cryfs-cli/Environment.h>

namespace po = boost::program_options;
namespace bf = boost::filesystem;
using namespace cryfs_unmount::program_options;
using cryfs::CryConfigConsole;
using cryfs::CryfsException;
using cryfs::ErrorCode;
using std::vector;
using std::cerr;
using std::endl;
using std::string;
using namespace cpputils::logging;

Parser::Parser(int argc, const char **argv)
	:_options(_argsToVector(argc, argv)) {
}

vector<string> Parser::_argsToVector(int argc, const char **argv) {
	vector<string> result;
	for (int i = 0; i < argc; ++i) {
		result.push_back(argv[i]);
	}
	return result;
}

ProgramOptions Parser::parse() const {
	po::variables_map vm = _parseOptionsOrShowHelp(_options);

	if (!vm.count("mount-dir")) {
		_showHelpAndExit("Please specify a mount directory.", ErrorCode::InvalidArguments);
	}
	bf::path mountDir = vm["mount-dir"].as<string>();

	return ProgramOptions(std::move(mountDir));
}

po::variables_map Parser::_parseOptionsOrShowHelp(const vector<string> &options) {
	try {
		return _parseOptions(options);
	}
	catch (const CryfsException& e) {
		// If CryfsException is thrown, we already know what's wrong.
		// Show usage information and pass through the exception, don't catch it.
		if (e.errorCode() != ErrorCode::Success) {
			_showHelp();
		}
		throw;
	}
	catch (const std::exception &e) {
		std::cerr << e.what() << std::endl;
		_showHelpAndExit("Invalid arguments", ErrorCode::InvalidArguments);
	}
}

po::variables_map Parser::_parseOptions(const vector<string> &options) {
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
		("version", "Show CryFS version number")
		;
	desc->add(options);
}

void Parser::_addPositionalOptionForBaseDir(po::options_description *desc, po::positional_options_description *positional) {
	positional->add("mount-dir", 1);
	po::options_description hidden("Hidden options");
	hidden.add_options()
		("mount-dir", po::value<string>(), "Mount directory")
		;
	desc->add(hidden);
}

void Parser::_showHelp() {
	cerr << "Usage: cryfs-unmount [mountPoint]\n";
	po::options_description desc;
	_addAllowedOptions(&desc);
	cerr << desc << endl;
}

[[noreturn]] void Parser::_showHelpAndExit(const std::string& message, ErrorCode errorCode) {
	_showHelp();
	throw CryfsException(message, errorCode);
}

[[noreturn]] void Parser::_showVersionAndExit() {
	// no need to show version because it was already shown in the CryFS header before parsing program options
	throw CryfsException("", ErrorCode::Success);
}
