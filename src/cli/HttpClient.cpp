// Base version taken from https://techoverflow.net/blog/2013/03/15/c-simple-http-download-using-libcurl-easy-api/
/**
 * HTTPDownloader.cpp
 *
 * A simple C++ wrapper for the libcurl easy API.
 *
 * Written by Uli Köhler (techoverflow.net)
 * Published under CC0 1.0 Universal (public domain)
 */
#include "HttpClient.h"
#include <sstream>
#include <iostream>
#include <curl/curl.h>
#include <curl/easy.h>

using boost::none;
using boost::optional;
using std::string;
using std::ostringstream;

size_t HttpClient::write_data(void *ptr, size_t size, size_t nmemb, ostringstream *stream) {
    stream->write((const char*)ptr, size*nmemb);
    return size * nmemb;
}

HttpClient::HttpClient() {
    curl = curl_easy_init();
}

HttpClient::~HttpClient() {
    curl_easy_cleanup(curl);
}

optional<string> HttpClient::get(const string& url) {
    curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
    // example.com is redirected, so we tell libcurl to follow redirection
    curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
    curl_easy_setopt(curl, CURLOPT_NOSIGNAL, 1); //Prevent "longjmp causes uninitialized stack frame" bug
    curl_easy_setopt(curl, CURLOPT_ACCEPT_ENCODING, "deflate");
    ostringstream out;
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, &HttpClient::write_data);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &out);
    // Perform the request, res will get the return code
    CURLcode res = curl_easy_perform(curl);
    // Check for errors
    if (res != CURLE_OK) {
        return none;
    }
    return out.str();
}
