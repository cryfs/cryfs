#include "FakeHttpClient.h"

using std::string;
using boost::optional;

namespace cpputils {
    FakeHttpClient::FakeHttpClient(): _sites() {
    }

    void FakeHttpClient::addWebsite(const string &url, const string &content) {
        _sites[url] = content;
    }

    string FakeHttpClient::get(const string &url, optional<long> timeoutMsec) {
        UNUSED(timeoutMsec);
        auto found = _sites.find(url);
        if (found == _sites.end()) {
			throw std::runtime_error("Website doesn't exist in FakeHttpClient.");
        }
        return found->second;
    }
}
