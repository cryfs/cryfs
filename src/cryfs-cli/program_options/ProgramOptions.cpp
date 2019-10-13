#include "ProgramOptions.h"
#include <cstring>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/system/path.h>

using namespace cryfs_cli::program_options;
using std::string;
using std::vector;
using boost::optional;
namespace bf = boost::filesystem;

ProgramOptions::ProgramOptions(bf::path baseDir, bf::path mountDir, optional<bf::path> configFile,
                               bool foreground, bool allowFilesystemUpgrade, bool allowReplacedFilesystem, optional<double> unmountAfterIdleMinutes,
                               optional<bf::path> logFile, optional<string> cipher,
                               optional<uint32_t> blocksizeBytes,
                               bool allowIntegrityViolations,
                               boost::optional<bool> missingBlockIsIntegrityViolation,
                               vector<string> fuseOptions)
    : _configFile(std::move(configFile)), _baseDir(bf::absolute(std::move(baseDir))), _mountDir(std::move(mountDir)),
      _mountDirIsDriveLetter(cpputils::path_is_just_drive_letter(_mountDir)),
	  _foreground(foreground),
	  _allowFilesystemUpgrade(allowFilesystemUpgrade), _allowReplacedFilesystem(allowReplacedFilesystem), _allowIntegrityViolations(allowIntegrityViolations),
      _cipher(std::move(cipher)), _blocksizeBytes(std::move(blocksizeBytes)), _unmountAfterIdleMinutes(std::move(unmountAfterIdleMinutes)),
      _missingBlockIsIntegrityViolation(std::move(missingBlockIsIntegrityViolation)), _logFile(std::move(logFile)),
      _fuseOptions(std::move(fuseOptions)) {
	if (!_mountDirIsDriveLetter) {
		_mountDir = bf::absolute(std::move(_mountDir));
	}
}

const bf::path &ProgramOptions::baseDir() const {
    return _baseDir;
}

const bf::path &ProgramOptions::mountDir() const {
    return _mountDir;
}

bool ProgramOptions::mountDirIsDriveLetter() const {
	return _mountDirIsDriveLetter;
}

const optional<bf::path> &ProgramOptions::configFile() const {
    return _configFile;
}

bool ProgramOptions::foreground() const {
    return _foreground;
}

bool ProgramOptions::allowFilesystemUpgrade() const {
  return _allowFilesystemUpgrade;
}

const optional<double> &ProgramOptions::unmountAfterIdleMinutes() const {
    return _unmountAfterIdleMinutes;
}

const optional<bf::path> &ProgramOptions::logFile() const {
    return _logFile;
}

const optional<string> &ProgramOptions::cipher() const {
    return _cipher;
}

const optional<uint32_t> &ProgramOptions::blocksizeBytes() const {
    return _blocksizeBytes;
}

bool ProgramOptions::allowIntegrityViolations() const {
    return _allowIntegrityViolations;
}

bool ProgramOptions::allowReplacedFilesystem() const {
    return _allowReplacedFilesystem;
}

const optional<bool> &ProgramOptions::missingBlockIsIntegrityViolation() const {
    return _missingBlockIsIntegrityViolation;
}

const vector<string> &ProgramOptions::fuseOptions() const {
    return _fuseOptions;
}
