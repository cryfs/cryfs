// Base version taken from https://techoverflow.net/blog/2013/03/15/c-simple-http-download-using-libcurl-easy-api/
/**
 * HTTPDownloader.hpp
 *
 * A simple C++ wrapper for the libcurl easy API.
 *
 * Written by Uli Köhler (techoverflow.net)
 * Published under CC0 1.0 Universal (public domain)
 */
#pragma once
#ifndef MESSMER_CRYFS_SRC_CLI_HTTPCLIENT_HPP
#define MESSMER_CRYFS_SRC_CLI_HTTPCLIENT_HPP

#include <string>
#include <boost/optional.hpp>
#include <messmer/cpp-utils/macros.h>

//TODO Test

/**
 * A non-threadsafe simple libcURL-easy based HTTP downloader
 */
class HttpClient final {
public:
    HttpClient();
    ~HttpClient();
    /**
     * Download a file using HTTP GET and store in in a std::string
     * @param url The URL to download
     * @return The download result
     */
    boost::optional<std::string> get(const std::string& url);
private:
    void* curl;

    static size_t write_data(void *ptr, size_t size, size_t nmemb, std::ostringstream *stream);

    DISALLOW_COPY_AND_ASSIGN(HttpClient);
};

#endif