#pragma once
#ifndef MESSMER_CPPUTILS_NETWORK_HTTPCLIENT_HPP
#define MESSMER_CPPUTILS_NETWORK_HTTPCLIENT_HPP

#include "HttpClient.h"
#include "../macros.h"

namespace cpputils {

    class CurlHttpClient final : public HttpClient {
    public:
        CurlHttpClient();

        ~CurlHttpClient();

        boost::optional <std::string> get(const std::string &url, boost::optional<long> timeoutMsec = boost::none) override;

    private:
        void *curl;

        static size_t write_data(void *ptr, size_t size, size_t nmemb, std::ostringstream *stream);

        DISALLOW_COPY_AND_ASSIGN(CurlHttpClient);
    };

}

#endif
