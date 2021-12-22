#include "subprocess.h"
#include <cstdio>
#include <stdexcept>
#include <cerrno>
#include <array>
#include <boost/process.hpp>
#include <boost/asio.hpp>

using std::string;
using std::vector;

namespace bp = boost::process;
namespace bf = boost::filesystem;
namespace ba = boost::asio;
namespace bs = boost::system;

#if defined(_MSC_VER)
constexpr auto PIPE_CLOSED = ba::error::broken_pipe;
#else
constexpr auto PIPE_CLOSED = ba::error::eof;
#endif

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

		class OutputPipeHandler final
		{
		public:
			explicit OutputPipeHandler(ba::io_context* ctx)
			: vOut_(128 * 1024)
			, buffer_(ba::buffer(vOut_))
			, pipe_(*ctx)
			, output_() {
			}

			void async_read()
			{
				std::function<void(const bs::error_code & ec, std::size_t n)> onOutput;
				onOutput = [&](const bs::error_code & ec, size_t n)
				{
					output_.reserve(output_.size() + n);
					output_.insert(output_.end(), vOut_.begin(), vOut_.begin() + n);
					if (ec) {
						if (ec != PIPE_CLOSED) {
							throw SubprocessError(std::string() + "Error getting output from subprocess. Error code: " + std::to_string(ec.value()) + " : " + ec.message());
						}
					} else {
						ba::async_read(pipe_, buffer_, onOutput);
					}
				};
				ba::async_read(pipe_, buffer_, onOutput);
			}

			bp::async_pipe& pipe()
			{
				return pipe_;
			}

			std::string output() &&
			{
				return std::move(output_);
			}


		private:
			std::vector<char> vOut_;
			ba::mutable_buffer buffer_;
			bp::async_pipe pipe_;
			std::string output_;
		};

		class InputPipeHandler final
		{
		public:
			explicit InputPipeHandler(ba::io_context* ctx, const std::string& input)
			: input_(input)
			, buffer_(ba::buffer(input_))
			, pipe_(*ctx) {

			}

			bp::async_pipe& pipe()
			{
				return pipe_;
			}

			void async_write()
			{
				ba::async_write(pipe_, buffer_, 
					[&](const bs::error_code & ec, std::size_t /*n*/) 
					{
						if (ec) {
							throw SubprocessError(std::string() + "Error sending input to subprocess. Error code: " + std::to_string(ec.value()) + " : " + ec.message());
						}
						pipe_.async_close();
					}
				);
			}
		private:
			const std::string& input_;
			ba::const_buffer buffer_;
			bp::async_pipe pipe_;
		};
	}

	SubprocessResult Subprocess::call(const char *command, const vector<string> &args, const string &input)
	{
		return call(_find_executable(command), args, input);
	}

	SubprocessResult Subprocess::check_call(const char *command, const vector<string> &args, const string& input)
	{
		return check_call(_find_executable(command), args, input);
	}

	SubprocessResult Subprocess::call(const bf::path& executable, const vector<string>& args, const string& input)
	{
		if (!bf::exists(executable))
		{
			throw std::runtime_error("Tried to run executable " + executable.string() + " but didn't find it");
		}

		// Process I/O needs to use the async API to avoid deadlocks, see
		// - https://www.boost.org/doc/libs/1_78_0/doc/html/boost_process/faq.html
		// - Code taken from https://www.py4u.net/discuss/97014 and modified

		ba::io_context ctx;

		OutputPipeHandler stdout_handler(&ctx);
		OutputPipeHandler stderr_handler(&ctx);
		InputPipeHandler stdin_handler(&ctx, input);

		bp::child child(
			bp::exe = executable.string(),
			bp::args(args),
			bp::std_out > stdout_handler.pipe(),
			bp::std_err > stderr_handler.pipe(),
			bp::std_in < stdin_handler.pipe()
		);

		stdin_handler.async_write();
		stdout_handler.async_read();
		stderr_handler.async_read();

		ctx.run();

		child.wait();

		return SubprocessResult{
			std::move(stdout_handler).output(),
			std::move(stderr_handler).output(),
			child.exit_code(),
		};
	}

	SubprocessResult Subprocess::check_call(const bf::path &executable, const vector<string> &args, const string& input)
	{
		auto result = call(executable, args, input);
		if (result.exitcode != 0)
		{
			throw SubprocessError("Subprocess \"" + executable.string() + "\" exited with code " + std::to_string(result.exitcode));
		}
		return result;
	}

}
