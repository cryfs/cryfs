#ifndef MESSMER_CPPUTILS_NETWORK_HTTPCLIENT_H
#define MESSMER_CPPUTILS_NETWORK_HTTPCLIENT_H

#include <string>
#include <boost/optional.hpp>

namespace cpputils {
    class HttpClient {
    public:
        virtual ~HttpClient() {}

        virtual boost::optional<std::string> get(const std::string& url) = 0;
    };
};

#endif
