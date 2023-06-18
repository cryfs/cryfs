#ifndef MESSMER_CPPUTILS_NETWORK_FAKEHTTPCLIENT_H
#define MESSMER_CPPUTILS_NETWORK_FAKEHTTPCLIENT_H

#include "HttpClient.h"
#include "../macros.h"
#include <map>

namespace cpputils {

    class FakeHttpClient final : public HttpClient {
    public:
        FakeHttpClient();

        void addWebsite(const std::string &url, const std::string &content);

        std::string get(const std::string &url, boost::optional<long> timeoutMsec = boost::none) override;

    private:
        std::map<std::string, std::string> _sites;

        DISALLOW_COPY_AND_ASSIGN(FakeHttpClient);
    };

}

#endif
