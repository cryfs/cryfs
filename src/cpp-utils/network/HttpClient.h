#ifndef MESSMER_CPPUTILS_NETWORK_HTTPCLIENT_H
#define MESSMER_CPPUTILS_NETWORK_HTTPCLIENT_H

#include <string>
#include <boost/optional.hpp>

namespace cpputils {
    class HttpClient {
    public:
        virtual ~HttpClient() {}

        virtual std::string get(const std::string& url, boost::optional<long> timeoutMsec = boost::none) = 0;
    };
};

#endif
