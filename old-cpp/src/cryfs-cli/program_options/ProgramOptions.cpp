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
                               bool foreground, bool allowFilesystemUpgrade, bool allowReplacedFilesystem, 
                               bool createMissingBasedir, bool createMissingMountpoint,
                               optional<double> unmountAfterIdleMinutes,
                               optional<bf::path> logFile, optional<string> cipher,
                               optional<uint32_t> blocksizeBytes,
                               bool allowIntegrityViolations,
                               boost::optional<bool> missingBlockIsIntegrityViolation,
                               vector<string> fuseOptions)
    : _baseDir(bf::absolute(std::move(baseDir))), _mountDir(std::move(mountDir)), _configFile(std::move(configFile)),
	  _foreground(foreground),
	  _allowFilesystemUpgrade(allowFilesystemUpgrade), _allowReplacedFilesystem(allowReplacedFilesystem),
      _createMissingBasedir(createMissingBasedir), _createMissingMountpoint(createMissingMountpoint),
      _unmountAfterIdleMinutes(std::move(unmountAfterIdleMinutes)), _logFile(std::move(logFile)),
      _cipher(std::move(cipher)), _blocksizeBytes(std::move(blocksizeBytes)),
      _allowIntegrityViolations(allowIntegrityViolations),
      _missingBlockIsIntegrityViolation(std::move(missingBlockIsIntegrityViolation)),
      _fuseOptions(std::move(fuseOptions)),
      _mountDirIsDriveLetter(cpputils::path_is_just_drive_letter(_mountDir)) {
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

bool ProgramOptions::createMissingBasedir() const {
    return _createMissingBasedir;
}

bool ProgramOptions::createMissingMountpoint() const {
    return _createMissingMountpoint;
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
