// Base version taken from https://techoverflow.net/blog/2013/03/15/c-simple-http-download-using-libcurl-easy-api/

#if !defined(_MSC_VER)

#include "CurlHttpClient.h"
#include <sstream>
#include <iostream>

using boost::none;
using boost::optional;
using std::string;
using std::ostringstream;
using std::mutex;
using std::unique_lock;

namespace cpputils {

	mutex CurlHttpClient::CurlInitializerRAII::_mutex;
	uint32_t CurlHttpClient::CurlInitializerRAII::_refcount = 0;

	CurlHttpClient::CurlInitializerRAII::CurlInitializerRAII() {
		unique_lock<mutex> lock(_mutex);
		if (0 == _refcount) {
			curl_global_init(CURL_GLOBAL_ALL);
		}
		_refcount += 1;
	}

	CurlHttpClient::CurlInitializerRAII::~CurlInitializerRAII() {
		unique_lock<mutex> lock(_mutex);
		_refcount -= 1;
		if (0 == _refcount) {
			curl_global_cleanup();
		}
	}

    size_t CurlHttpClient::write_data(void *ptr, size_t size, size_t nmemb, ostringstream *stream) {
        stream->write(static_cast<const char *>(ptr), size * nmemb);
        return size * nmemb;
    }

    CurlHttpClient::CurlHttpClient(): curlInitializer(), curl() {
        curl = curl_easy_init();
    }

    CurlHttpClient::~CurlHttpClient() {
        curl_easy_cleanup(curl);
    }

    string CurlHttpClient::get(const string &url, optional<long> timeoutMsec) {
        curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
        // example.com is redirected, so we tell libcurl to follow redirection
        curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
        curl_easy_setopt(curl, CURLOPT_NOSIGNAL, 1); //Prevent "longjmp causes uninitialized stack frame" bug
        curl_easy_setopt(curl, CURLOPT_ENCODING, "deflate");
        ostringstream out;
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, &CurlHttpClient::write_data);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &out);
        if (timeoutMsec != none) {
            curl_easy_setopt(curl, CURLOPT_TIMEOUT_MS, *timeoutMsec);
        }
        // Perform the request, res will get the return code
        CURLcode res = curl_easy_perform(curl);
        // Check for errors
        if (res != CURLE_OK) {
			throw std::runtime_error("Curl Error " + std::to_string(res) + ": " + curl_easy_strerror(res));
        }
        return out.str();
    }

}

#endif
