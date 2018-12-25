#if defined(_MSC_VER)

#include "WinHttpClient.h"
#include <sstream>
#include <iostream>
#include <cpp-utils/assert/assert.h>
#include <cpp-utils/data/Data.h>
#include <codecvt>
#include <Windows.h>
#include <Winhttp.h>
#include <VersionHelpers.h>

using boost::none;
using boost::optional;
using std::string;
using std::wstring;
using std::wstring_convert;
using std::ostringstream;

namespace cpputils {

	namespace {
		struct HttpHandleRAII final {
			HINTERNET handle;

			HttpHandleRAII(HINTERNET handle_) : handle(handle_) {}

			HttpHandleRAII(HttpHandleRAII&& rhs) : handle(rhs.handle) {
				rhs.handle = nullptr;
			}

			~HttpHandleRAII() {
				if (nullptr != handle) {
					BOOL success = WinHttpCloseHandle(handle);
					if (!success) {
						throw std::runtime_error("Error calling WinHttpCloseHandle. Error code: " + std::to_string(GetLastError()));
					}
				}
			}

			DISALLOW_COPY_AND_ASSIGN(HttpHandleRAII);
		};

		URL_COMPONENTS parse_url(const wstring &url) {
			URL_COMPONENTS result;
			result.dwStructSize = sizeof(result);
			// Declare fields we want. Setting a field to nullptr and the length to non-zero means the field will be returned.
			result.lpszScheme = nullptr;
			result.dwSchemeLength = 1;
			result.lpszHostName = nullptr;
			result.dwHostNameLength = 1;
			result.lpszUserName = nullptr;
			result.dwUserNameLength = 1;
			result.lpszPassword = nullptr;
			result.dwPasswordLength = 1;
			result.lpszUrlPath = nullptr;
			result.dwUrlPathLength = 1;
			result.lpszExtraInfo = nullptr;
			result.dwExtraInfoLength = 1;

			BOOL success = WinHttpCrackUrl(url.c_str(), url.size(), ICU_REJECT_USERPWD, &result);
			if (!success) {
				throw std::runtime_error("Error parsing url '" + wstring_convert<std::codecvt_utf8_utf16<wchar_t>>().to_bytes(url) + "'. Error code: " + std::to_string(GetLastError()));
			}

			return result;
		}

		INTERNET_PORT get_port_from_url(const URL_COMPONENTS& parsedUrl) {
			wstring scheme_str(parsedUrl.lpszScheme, parsedUrl.dwSchemeLength);
			string s_(wstring_convert < std::codecvt_utf8_utf16<wchar_t>>().to_bytes(scheme_str));
			if (parsedUrl.nScheme == INTERNET_SCHEME_HTTP) {
				ASSERT(scheme_str == L"http", "Scheme mismatch");
				if (parsedUrl.nPort != 80) {
					throw std::runtime_error("We don't support non-default ports");
				}
				return INTERNET_DEFAULT_HTTP_PORT;
			}
			else if (parsedUrl.nScheme == INTERNET_SCHEME_HTTPS) {
				ASSERT(scheme_str == L"https", "Scheme mismatch");
				if (parsedUrl.nPort != 443) {
					throw std::runtime_error("We don't support non-default ports");
				}
				return INTERNET_DEFAULT_HTTPS_PORT;
			}
			else {
				throw std::runtime_error("Unsupported scheme: " + wstring_convert<std::codecvt_utf8_utf16<wchar_t>>().to_bytes(scheme_str));
			}
		}

		class Request final {
		public:
			Request(HttpHandleRAII request) : request_(std::move(request)) {}

			void set_redirect_policy(DWORD redirectPolicy) {
				BOOL success = WinHttpSetOption(request_.handle, WINHTTP_OPTION_REDIRECT_POLICY, &redirectPolicy, sizeof(redirectPolicy));
				if (!success) {
					throw std::runtime_error("Error calling WinHttpSetOption. Error code: " + std::to_string(GetLastError()));
				}
			}

			void set_timeouts(long timeoutMsec) {
				// TODO Timeout should be a total timeout, not per step as we're doing it here.
				BOOL success = WinHttpSetTimeouts(request_.handle, timeoutMsec, timeoutMsec, timeoutMsec, timeoutMsec);
				if (!success) {
					throw std::runtime_error("Error calling WinHttpSetTimeouts. Error code: " + std::to_string(GetLastError()));
				}
			}

			void send() {
				BOOL success = WinHttpSendRequest(request_.handle, WINHTTP_NO_ADDITIONAL_HEADERS, 0, WINHTTP_NO_REQUEST_DATA, 0, 0, 0);
				if (!success) {
					throw std::runtime_error("Error calling WinHttpSendRequest. Error code: " + std::to_string(GetLastError()));
				}
			}

			void wait_for_response() {
				BOOL success = WinHttpReceiveResponse(request_.handle, nullptr);
				if (!success) {
					throw std::runtime_error("Error calling WinHttpReceiveResponse. Error code: " + std::to_string(GetLastError()));
				}
			}

			DWORD get_status_code() {
				DWORD statusCode;
				DWORD statusCodeSize = sizeof(statusCode);
				BOOL success = WinHttpQueryHeaders(request_.handle, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_HEADER_NAME_BY_INDEX, &statusCode, &statusCodeSize, WINHTTP_NO_HEADER_INDEX);
				if (!success) {
					throw std::runtime_error("Eror calling WinHttpQueryHeaders. Error code: " + std::to_string(GetLastError()));
				}
				return statusCode;
			}

			string read_response() {
				ostringstream result;

				while (true) {
					DWORD size = num_bytes_readable();
					if (size == 0) {
						break;
					}

					cpputils::Data buffer(size + 1);
					buffer.FillWithZeroes();

					DWORD num_read;
					BOOL success = WinHttpReadData(request_.handle, buffer.data(), buffer.size(), &num_read);
					if (!success) {
						throw std::runtime_error("Error calling WinHttpReadData. Error code: " + std::to_string(GetLastError()));
					}
					ASSERT(0 != num_read, "Weird behavior of WinHttpReadData.It should never read zero bytes since WinHttpQueryDataAvailable said there are bytes readable.");

					result.write(reinterpret_cast<char*>(buffer.data()), num_read);
					ASSERT(result.good(), "Error writing to ostringstream");
				}

				return result.str();
			}

		private:
			DWORD num_bytes_readable() {
				DWORD result;
				BOOL success = WinHttpQueryDataAvailable(request_.handle, &result);
				if (!success) {
					throw std::runtime_error("Error calling WinHttpQueryDataAvailable. Error code: " + std::to_string(GetLastError()));
				}
				return result;
			}

			HttpHandleRAII request_;
		};

		struct Connection final {
		public:
			Connection(HttpHandleRAII connection) : connection_(std::move(connection)) {}

			Request create_request(const URL_COMPONENTS& parsedUrl) {
				const INTERNET_PORT port = get_port_from_url(parsedUrl);
				const wstring path = wstring(parsedUrl.lpszUrlPath, parsedUrl.dwUrlPathLength) + wstring(parsedUrl.lpszExtraInfo, parsedUrl.dwExtraInfoLength);
				const DWORD flags = (port == INTERNET_DEFAULT_HTTPS_PORT) ? WINHTTP_FLAG_SECURE : 0;

				HttpHandleRAII request_handle(WinHttpOpenRequest(connection_.handle, L"GET", path.c_str(), nullptr, WINHTTP_NO_REFERER, WINHTTP_DEFAULT_ACCEPT_TYPES, flags));
				if (nullptr == request_handle.handle) {
					throw std::runtime_error("Error calling WinHttpOpenRequest. Error code: " + std::to_string(GetLastError()));
				}
				return Request(std::move(request_handle));
			}

		private:
			HttpHandleRAII connection_;
		};
	}

	struct WinHttpSession final {
	public:
		WinHttpSession(HttpHandleRAII session) : session_(std::move(session)) {}

		Connection create_connection(const URL_COMPONENTS& parsedUrl) {
			const INTERNET_PORT port = get_port_from_url(parsedUrl);
			const wstring host(parsedUrl.lpszHostName, parsedUrl.dwHostNameLength);

			HttpHandleRAII connection_handle = WinHttpConnect(session_.handle, host.c_str(), port, 0);
			if (nullptr == connection_handle.handle) {
				throw std::runtime_error("Error calling WinHttpConnect. Error code: " + std::to_string(GetLastError()));
			}

			return Connection(std::move(connection_handle));
		}

	private:
		HttpHandleRAII session_;
	};

	namespace {
		cpputils::unique_ref<WinHttpSession> create_session() {
      const DWORD dwAccessType = IsWindows8Point1OrGreater() ? WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY : WINHTTP_ACCESS_TYPE_DEFAULT_PROXY;
			HttpHandleRAII session_handle = WinHttpOpen(L"cpputils::HttpClient", dwAccessType, WINHTTP_NO_PROXY_NAME, WINHTTP_NO_PROXY_BYPASS, 0);
			if(nullptr == session_handle.handle) {
				throw std::runtime_error("Error calling WinHttpOpen. Error code: " + std::to_string(GetLastError()));
			}

			return cpputils::make_unique_ref<WinHttpSession>(std::move(session_handle));
		}
	}

	WinHttpClient::WinHttpClient() : session_(create_session()) {}

	WinHttpClient::~WinHttpClient() {}

	string WinHttpClient::get(const string &url, optional<long> timeoutMsec) {
		wstring wurl = wstring_convert<std::codecvt_utf8_utf16<wchar_t>>().from_bytes(url);
		const URL_COMPONENTS parsedUrl = parse_url(wurl);

		ASSERT(parsedUrl.dwUserNameLength == 0, "Authentication not supported");
		ASSERT(parsedUrl.dwPasswordLength == 0, "Authentication not supported");

		Connection connection = session_->create_connection(parsedUrl);
		Request request = connection.create_request(parsedUrl);

		// allow redirects but not from https to http
		request.set_redirect_policy(WINHTTP_OPTION_REDIRECT_POLICY_DISALLOW_HTTPS_TO_HTTP);

		if (timeoutMsec != none) {
			request.set_timeouts(*timeoutMsec);
		}

		request.send();
		request.wait_for_response();

		DWORD statusCode = request.get_status_code();
		if (statusCode != HTTP_STATUS_OK) {
			throw std::runtime_error("HTTP Server returned unsupported status code: " + std::to_string(statusCode));
		}

		return request.read_response();
	}

}

#endif
