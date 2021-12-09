#include "subprocess.h"
#include <cstdio>
#include <stdexcept>
#include <cerrno>
#include <array>
#include <boost/process.hpp>

using std::string;
using std::vector;

namespace bp = boost::process;
namespace bf = boost::filesystem;

namespace cpputils
{
	namespace
	{
		bf::path _find_executable(const char *command)
		{
			bf::path executable = bp::search_path(command);
			if (executable == "")
			{
				throw std::runtime_error("Tried to run command " + std::string(command) + " but didn't find it in the PATH");
			}
			return executable;
		}
	}

	SubprocessResult Subprocess::call(const char *command, const vector<string> &args)
	{
		return call(_find_executable(command), args);
	}

	SubprocessResult Subprocess::check_call(const char *command, const vector<string> &args)
	{
		return check_call(_find_executable(command), args);
	}

	SubprocessResult Subprocess::call(const bf::path &executable, const vector<string> &args)
	{
		if (!bf::exists(executable))
		{
			throw std::runtime_error("Tried to run executable " + executable.string() + " but didn't find it");
		}

		bp::ipstream child_stdout;
		bp::ipstream child_stderr;
		bp::child child = bp::child(bp::exe = executable.string(), bp::std_out > child_stdout, bp::std_err > child_stderr, bp::args(args));
		if (!child.valid())
		{
			throw std::runtime_error("Error starting subprocess " + executable.string() + ". Errno: " + std::to_string(errno));
		}

		child.join();

		string output_stdout = string(std::istreambuf_iterator<char>(child_stdout), {});
		string output_stderr = string(std::istreambuf_iterator<char>(child_stderr), {});

		return SubprocessResult{
			std::move(output_stdout),
			std::move(output_stderr),
			child.exit_code(),
		};
	}

	SubprocessResult Subprocess::check_call(const bf::path &executable, const vector<string> &args)
	{
		auto result = call(executable, args);
		if (result.exitcode != 0)
		{
			throw SubprocessError("Subprocess \"" + executable.string() + "\" exited with code " + std::to_string(result.exitcode));
		}
		return result;
	}

}
