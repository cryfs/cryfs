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
		std::string exception_code_string(DWORD exception_code) {
#define HANDLE_CODE(code) case code: return #code; 
			switch (exception_code) {
				// List of exception codes taken from https://docs.microsoft.com/en-us/windows/desktop/Debug/getexceptioncode
				HANDLE_CODE(EXCEPTION_ACCESS_VIOLATION)
				HANDLE_CODE(EXCEPTION_ARRAY_BOUNDS_EXCEEDED)
				HANDLE_CODE(EXCEPTION_BREAKPOINT)
				HANDLE_CODE(EXCEPTION_DATATYPE_MISALIGNMENT)
				HANDLE_CODE(EXCEPTION_FLT_DENORMAL_OPERAND)
				HANDLE_CODE(EXCEPTION_FLT_DIVIDE_BY_ZERO)
				HANDLE_CODE(EXCEPTION_FLT_INEXACT_RESULT)
				HANDLE_CODE(EXCEPTION_FLT_INVALID_OPERATION)
				HANDLE_CODE(EXCEPTION_FLT_OVERFLOW)
				HANDLE_CODE(EXCEPTION_FLT_STACK_CHECK)
				HANDLE_CODE(EXCEPTION_FLT_UNDERFLOW)
				HANDLE_CODE(EXCEPTION_GUARD_PAGE)
				HANDLE_CODE(EXCEPTION_ILLEGAL_INSTRUCTION)
				HANDLE_CODE(EXCEPTION_IN_PAGE_ERROR)
				HANDLE_CODE(EXCEPTION_INT_DIVIDE_BY_ZERO)
				HANDLE_CODE(EXCEPTION_INT_OVERFLOW)
				HANDLE_CODE(EXCEPTION_INVALID_DISPOSITION)
				HANDLE_CODE(EXCEPTION_INVALID_HANDLE)
				HANDLE_CODE(EXCEPTION_NONCONTINUABLE_EXCEPTION)
				HANDLE_CODE(EXCEPTION_PRIV_INSTRUCTION)
				HANDLE_CODE(EXCEPTION_SINGLE_STEP)
				HANDLE_CODE(EXCEPTION_STACK_OVERFLOW)
				HANDLE_CODE(STATUS_UNWIND_CONSOLIDATE)
			default:
				std::ostringstream str;
				str << "UNKNOWN_CODE(0x" << std::hex << exception_code << ")";
				return str.str();
			}
#undef HANDLE_CODE
		}

		struct SymInitializeRAII final {
			const HANDLE process;
			const bool success;

			SymInitializeRAII()
				: process(GetCurrentProcess())
				, success(::SymInitialize(process, NULL, TRUE)) {
			}

			~SymInitializeRAII() {
				::SymCleanup(process);
			}
		};

		std::string backtrace_to_string(CONTEXT* context_record) {
			std::ostringstream backtrace;

			SymInitializeRAII sym;
			if (!sym.success) {
				DWORD error = GetLastError();
				backtrace << "[Can't get backtrace. SymInitialize failed with error code " << std::dec << error << "]\n";
			} else {
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
					sym.process,
					GetCurrentThread(),
					&stack_frame,
					context_record,
					nullptr,
					&SymFunctionTableAccess64,
					&SymGetModuleBase64,
					nullptr)) {

					backtrace << "#" << (i++) << " ";

					DWORD64 displacement = 0;

					if (SymFromAddr(sym.process, (DWORD64)stack_frame.AddrPC.Offset, &displacement, symbol))
					{
						IMAGEHLP_MODULE64 moduleInfo;
						std::memset(&moduleInfo, 0, sizeof(IMAGEHLP_MODULE64));
						moduleInfo.SizeOfStruct = sizeof(moduleInfo);

						if (::SymGetModuleInfo64(sym.process, symbol->ModBase, &moduleInfo)) {
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
					if (::SymGetLineFromAddr64(sym.process, (DWORD64)stack_frame.AddrPC.Offset, &dwDisplacement, &line)) {
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

		namespace {
			bool our_top_level_handler_set = false;
			LPTOP_LEVEL_EXCEPTION_FILTER previous_top_level_handler = nullptr;
		}

		LONG WINAPI TopLevelExceptionHandler(PEXCEPTION_POINTERS pExceptionInfo)
		{
			std::string backtrace = backtrace_to_string(pExceptionInfo->ContextRecord);
			LOG(ERR, "Top level exception. Code: {}. Backtrace:\n{}", exception_code_string(pExceptionInfo->ExceptionRecord->ExceptionCode), backtrace);
			
			if (previous_top_level_handler != nullptr) {
				// There was already a top level exception handler set when we called showBacktraceOnCrash(). Call it.
				return (*previous_top_level_handler)(pExceptionInfo);
			} else {
				// previous_top_level_handler == nullptr means there was no top level exception handler set when we called showBacktraceOnCrash()
				// so there's nothing else we need to call.
				return EXCEPTION_CONTINUE_SEARCH;
			}
		}
	}

	std::string backtrace() {
		CONTEXT context;
		memset(&context, 0, sizeof(CONTEXT));
		context.ContextFlags = CONTEXT_FULL;
		RtlCaptureContext(&context);
		return backtrace_to_string(&context);
	}

	void showBacktraceOnCrash() {
		if (!our_top_level_handler_set) {
			previous_top_level_handler = SetUnhandledExceptionFilter(TopLevelExceptionHandler);
			our_top_level_handler_set = true;
		}
	}

}

#endif
