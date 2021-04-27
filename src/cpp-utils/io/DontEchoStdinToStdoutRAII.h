#pragma once
#ifndef MESSMER_CPPUTILS_IO_DONTECHOSTDINTOSTDOUTRAII_H
#define MESSMER_CPPUTILS_IO_DONTECHOSTDINTOSTDOUTRAII_H

#include <cpp-utils/pointer/unique_ref.h>
#include "../macros.h"

/**
 * If you create an instance of this class in your scope, then any user input from stdin
 * won't be echoed back to stdout until the instance leaves the scope.
 * This can be very handy for password inputs where you don't want the password to be visible on screen.
 */

namespace cpputils
{

	namespace details
	{
		class DontEchoStdinToStdoutRAII_;
	}

	class DontEchoStdinToStdoutRAII final
	{
	public:
		DontEchoStdinToStdoutRAII();
		~DontEchoStdinToStdoutRAII();

	private:
		cpputils::unique_ref<details::DontEchoStdinToStdoutRAII_> raii;

		DISALLOW_COPY_AND_ASSIGN(DontEchoStdinToStdoutRAII);
	};

}

#endif
