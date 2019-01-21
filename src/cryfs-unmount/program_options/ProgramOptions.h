#pragma once
#ifndef MESSMER_CRYFSUNMOUNT_PROGRAMOPTIONS_PROGRAMOPTIONS_H
#define MESSMER_CRYFSUNMOUNT_PROGRAMOPTIONS_PROGRAMOPTIONS_H

#include <vector>
#include <string>
#include <boost/optional.hpp>
#include <cpp-utils/macros.h>
#include <boost/filesystem.hpp>

namespace cryfs_unmount {
	namespace program_options {
		class ProgramOptions final {
		public:
			ProgramOptions(boost::filesystem::path mountDir);
			ProgramOptions(ProgramOptions &&rhs) = default;

			const boost::filesystem::path &mountDir() const;
			bool mountDirIsDriveLetter() const;

		private:
			boost::filesystem::path _mountDir;
			bool _mountDirIsDriveLetter;

			DISALLOW_COPY_AND_ASSIGN(ProgramOptions);
		};
	}
}

#endif
