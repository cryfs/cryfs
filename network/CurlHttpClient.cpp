// Base version taken from https://techoverflow.net/blog/2013/03/15/c-simple-http-download-using-libcurl-easy-api/

#include "CurlHttpClient.h"
#include <sstream>
#include <iostream>
#include <curl/curl.h>
#include <curl/easy.h>

using boost::none;
using boost::optional;
using std::string;
using std::ostringstream;

namespace cpputils {

    size_t CurlHttpClient::write_data(void *ptr, size_t size, size_t nmemb, ostringstream *stream) {
        stream->write((const char *) ptr, size * nmemb);
        return size * nmemb;
    }

    CurlHttpClient::CurlHttpClient() {
        curl = curl_easy_init();
    }

    CurlHttpClient::~CurlHttpClient() {
        curl_easy_cleanup(curl);
    }

    optional <string> CurlHttpClient::get(const string &url, optional<long> timeoutMsec) {
        curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
        // example.com is redirected, so we tell libcurl to follow redirection
        curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
        curl_easy_setopt(curl, CURLOPT_NOSIGNAL, 1); //Prevent "longjmp causes uninitialized stack frame" bug
        curl_easy_setopt(curl, CURLOPT_ACCEPT_ENCODING, "deflate");
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
            return none;
        }
        return out.str();
    }

}
