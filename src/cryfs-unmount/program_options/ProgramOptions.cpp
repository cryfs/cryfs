#include "ProgramOptions.h"
#include <cstring>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/system/path.h>

using namespace cryfs_unmount::program_options;
using std::string;
namespace bf = boost::filesystem;

ProgramOptions::ProgramOptions(bf::path mountDir)
	:_mountDir(std::move(mountDir)),
	 _mountDirIsDriveLetter(cpputils::path_is_just_drive_letter(_mountDir))
{
	if (!_mountDirIsDriveLetter) {
		_mountDir = bf::absolute(std::move(_mountDir));
	}
}

const bf::path &ProgramOptions::mountDir() const {
	return _mountDir;
}

bool ProgramOptions::mountDirIsDriveLetter() const {
	return _mountDirIsDriveLetter;
}
