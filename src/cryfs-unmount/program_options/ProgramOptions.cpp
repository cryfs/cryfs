#include "ProgramOptions.h"
#include <boost/filesystem/operations.hpp>
#include <boost/filesystem/path.hpp>
#include <cpp-utils/system/path.h>
#include <string>
#include <utility>

using namespace cryfs_unmount::program_options;
using std::string;
namespace bf = boost::filesystem;

ProgramOptions::ProgramOptions(bf::path mountDir, bool immediate)
	: _mountDir(std::move(mountDir)),
	  _mountDirIsDriveLetter(cpputils::path_is_just_drive_letter(_mountDir)),
	  _immediate(immediate)
{
	if (!_mountDirIsDriveLetter)
	{
		_mountDir = bf::absolute(std::move(_mountDir));
	}
}

const bf::path &ProgramOptions::mountDir() const
{
	return _mountDir;
}

bool ProgramOptions::mountDirIsDriveLetter() const
{
	return _mountDirIsDriveLetter;
}

bool ProgramOptions::immediate() const
{
	return _immediate;
}
