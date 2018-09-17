#pragma once
#ifndef MESSMER_CPPUTILS_NETWORK_CURLHTTPCLIENT_HPP
#define MESSMER_CPPUTILS_NETWORK_CURLHTTPCLIENT_HPP

#if !defined(_MSC_VER)

#include "HttpClient.h"
#include "../macros.h"
#include <mutex>
#include <curl/curl.h>

namespace cpputils {

    class CurlHttpClient final : public HttpClient {
    public:
        CurlHttpClient();

        ~CurlHttpClient();

        std::string get(const std::string &url, boost::optional<long> timeoutMsec = boost::none) override;

    private:
		// When the first object of this class is created, it will initialize curl using curl_global_init().
		// When the last object is destroyed, it will deinitialize curl using curl_global_cleanup().
		class CurlInitializerRAII final {
		public:
			CurlInitializerRAII();
			~CurlInitializerRAII();
		private:
			static std::mutex _mutex;
			static uint32_t _refcount;

			DISALLOW_COPY_AND_ASSIGN(CurlInitializerRAII);
		};

        CurlInitializerRAII curlInitializer;
        CURL *curl;

        static size_t write_data(void *ptr, size_t size, size_t nmemb, std::ostringstream *stream);

        DISALLOW_COPY_AND_ASSIGN(CurlHttpClient);
    };

}

#endif

#endif
