#if defined(_MSC_VER)

#include "backtrace.h"
#include <string>
#include <sstream>
#include "../logging/logging.h"
#include <Dbghelp.h>

using std::string;
using std::ostringstream;
using namespace cpputils::logging;

namespace cpputils {

	namespace {
		std::string backtrace_to_string(CONTEXT* context_record) {
			std::ostringstream backtrace;

			HANDLE process = GetCurrentProcess();
			if (!::SymInitialize(process, NULL, TRUE)) {
				DWORD error = GetLastError();
				backtrace << "[Can't get backtrace. SymInitialize failed with error code " << std::dec << error << "]\n";
			}
			else {
				// Initialize stack walking.
				STACKFRAME64 stack_frame;
				memset(&stack_frame, 0, sizeof(stack_frame));
#if defined(_WIN64)
				int machine_type = IMAGE_FILE_MACHINE_AMD64;
				stack_frame.AddrPC.Offset = context_record->Rip;
				stack_frame.AddrFrame.Offset = context_record->Rbp;
				stack_frame.AddrStack.Offset = context_record->Rsp;
#else
				int machine_type = IMAGE_FILE_MACHINE_I386;
				stack_frame.AddrPC.Offset = context_record->Eip;
				stack_frame.AddrFrame.Offset = context_record->Ebp;
				stack_frame.AddrStack.Offset = context_record->Esp;
#endif
				stack_frame.AddrPC.Mode = AddrModeFlat;
				stack_frame.AddrFrame.Mode = AddrModeFlat;
				stack_frame.AddrStack.Mode = AddrModeFlat;

				auto symbol_storage = std::make_unique<char[]>(sizeof(SYMBOL_INFO) + MAX_SYM_NAME * sizeof(TCHAR));
				PSYMBOL_INFO symbol = (PSYMBOL_INFO)symbol_storage.get();
				symbol->SizeOfStruct = sizeof(SYMBOL_INFO);
				symbol->MaxNameLen = MAX_SYM_NAME;

				int i = 0;

				while (StackWalk64(machine_type,
					GetCurrentProcess(),
					GetCurrentThread(),
					&stack_frame,
					context_record,
					nullptr,
					&SymFunctionTableAccess64,
					&SymGetModuleBase64,
					nullptr)) {

					backtrace << "#" << (i++) << " ";

					DWORD64 displacement = 0;

					if (SymFromAddr(process, (DWORD64)stack_frame.AddrPC.Offset, &displacement, symbol))
					{
						IMAGEHLP_MODULE64 moduleInfo;
						std::memset(&moduleInfo, 0, sizeof(IMAGEHLP_MODULE64));
						moduleInfo.SizeOfStruct = sizeof(moduleInfo);

						if (::SymGetModuleInfo64(process, symbol->ModBase, &moduleInfo)) {
							backtrace << moduleInfo.ModuleName << ":";
						}
						backtrace << "0x" << std::hex << (DWORD64)stack_frame.AddrPC.Offset << ": ";

						backtrace << symbol->Name << " + 0x" << std::hex << static_cast<int64_t>(displacement);
					}
					else {
						DWORD error = GetLastError();
						backtrace << std::hex << (DWORD64)stack_frame.AddrPC.Offset << ": [can't get symbol. SymFromAddr failed with error code " << std::dec << error << "]";
					}

					DWORD dwDisplacement;
					IMAGEHLP_LINE64 line;
					SymSetOptions(SYMOPT_LOAD_LINES);
					line.SizeOfStruct = sizeof(IMAGEHLP_LINE64);
					if (::SymGetLineFromAddr64(process, (DWORD64)stack_frame.AddrPC.Offset, &dwDisplacement, &line)) {
						backtrace << " at " << line.FileName << ":" << std::dec << line.LineNumber;
					}
					else {
						DWORD error = GetLastError();
						backtrace << " at [file/line unavailable, SymGetLineFromAddr64 failed with error code " << std::dec << error << "]";
					}
					backtrace << "\n";
				}
			}

			return backtrace.str();
		}


		LONG WINAPI TopLevelExceptionHandler(PEXCEPTION_POINTERS pExceptionInfo)
		{
			std::string backtrace = backtrace_to_string(pExceptionInfo->ContextRecord);
			LOG(ERROR, "Top level exception. Backtrace:\n{}", backtrace);

			return EXCEPTION_CONTINUE_SEARCH;
		}
	}

	std::string backtrace() {
		CONTEXT context;
		memset(&context, 0, sizeof(CONTEXT));
		context.ContextFlags = CONTEXT_FULL;
		RtlCaptureContext(&context);
		return backtrace_to_string(&context);
	}

	void showBacktraceOnSigSegv() {
		SetUnhandledExceptionFilter(TopLevelExceptionHandler);
	}

}

#endif
