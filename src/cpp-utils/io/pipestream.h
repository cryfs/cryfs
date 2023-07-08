// Original version taken under MIT licence from http://stackoverflow.com/a/12413298/829568 and modified.
// ----------------------------------------------------------------------------
//  Copyright (C) 2013 Dietmar Kuehl http://www.dietmar-kuehl.de
//
//  Permission is hereby granted, free of charge, to any person
//  obtaining a copy of this software and associated documentation
//  files (the "Software"), to deal in the Software without restriction,
//  including without limitation the rights to use, copy, modify,
//  merge, publish, distribute, sublicense, and/or sell copies of
//  the Software, and to permit persons to whom the Software is
//  furnished to do so, subject to the following conditions:
//
//  The above copyright notice and this permission notice shall be
//  included in all copies or substantial portions of the Software.
//
//  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
//  EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
//  OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
//  NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
//  HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
//  WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
//  FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
//  OTHER DEALINGS IN THE SOFTWARE.
// ----------------------------------------------------------------------------
#pragma once
#ifndef MESSMER_CPPUTILS_PIPESTREAM_H
#define MESSMER_CPPUTILS_PIPESTREAM_H

/**
 * This is a class that implements a pipe for std::ostream/std::istream.
 * You can in one thread write to std::ostream and read that data (blocking) from an std::istream.
 * Reading and writing can happen in different threads.
 *
 * Use as follows:
 *  pipestream pipe;
 *  std::istream istream(&pipe);
 *  std::ostream ostream(&pipe);
 *  istream << "Data";
 *  ostream >> ...
 */

#include <algorithm>
#include <condition_variable>
#include <iostream>
#include <mutex>
#include <stdexcept>
#include <streambuf>
#include <string>
#include <thread>
#include "../macros.h"

//TODO Add test cases

namespace cpputils {

    class pipestream final : public std::streambuf {
    private:
        typedef std::streambuf::traits_type traits_type;
        typedef std::string::size_type string_size_t;

        std::mutex d_mutex;
        std::condition_variable d_condition;
        std::string d_out;
        std::string d_in;
        std::string d_tmp;
        char *d_current;
        bool d_closed;

    public:
        pipestream(string_size_t out_size = 16, string_size_t in_size = 64)
        : d_mutex()
        ,  d_condition()
        ,  d_out(std::max(string_size_t(1), out_size), ' ')
        , d_in(std::max(string_size_t(1), in_size), ' ')
        , d_tmp(std::max(string_size_t(1), in_size), ' ')
        , d_current(&this->d_tmp[0])
        , d_closed(false)
        {
            this->setp(&this->d_out[0], &this->d_out[0] + this->d_out.size() - 1);
            this->setg(&this->d_in[0], &this->d_in[0], &this->d_in[0]);
        }

        void close() {
            {
                std::unique_lock <std::mutex> lock(this->d_mutex);
                this->d_closed = true;
                while (this->pbase() != this->pptr()) {
                    this->internal_sync(lock);
                }
            }
            this->d_condition.notify_all();
        }

    private:
        int_type underflow() override {
            if (this->gptr() == this->egptr()) {
                std::unique_lock <std::mutex> lock(this->d_mutex);
                while (&this->d_tmp[0] == this->d_current && !this->d_closed) {
                    this->d_condition.wait(lock);
                }
                if (&this->d_tmp[0] != this->d_current) {
                    const std::streamsize size(this->d_current - &this->d_tmp[0]);
                    traits_type::copy(this->eback(), &this->d_tmp[0],
                                      this->d_current - &this->d_tmp[0]);
                    this->setg(this->eback(), this->eback(), this->eback() + size);
                    this->d_current = &this->d_tmp[0];
                    this->d_condition.notify_one();
                }
            }
            return this->gptr() == this->egptr()
                   ? traits_type::eof()
                   : traits_type::to_int_type(*this->gptr());
        }

        int_type overflow(int_type c) override {
            std::unique_lock <std::mutex> lock(this->d_mutex);
            if (!traits_type::eq_int_type(c, traits_type::eof())) {
                *this->pptr() = traits_type::to_char_type(c);
                this->pbump(1);
            }
            return this->internal_sync(lock)
                   ? traits_type::eof()
                   : traits_type::not_eof(c);
        }

        int sync() override {
            std::unique_lock <std::mutex> lock(this->d_mutex);
            return this->internal_sync(lock);
        }

        int internal_sync(std::unique_lock <std::mutex> &lock) {
            char *end(&this->d_tmp[0] + this->d_tmp.size());
            while (this->d_current == end && !this->d_closed) {
                this->d_condition.wait(lock);
            }
            if (this->d_current != end) {
                const std::streamsize size(std::min(end - d_current,
                                              this->pptr() - this->pbase()));
                traits_type::copy(d_current, this->pbase(), size);
                this->d_current += size;
                const std::streamsize remain((this->pptr() - this->pbase()) - size);
                traits_type::move(this->pbase(), this->pptr(), remain);
                this->setp(this->pbase(), this->epptr());
                this->pbump(static_cast<int>(remain));
                this->d_condition.notify_one();
                return 0;
            }
            return traits_type::eof();
        }

        DISALLOW_COPY_AND_ASSIGN(pipestream);
    };

}

#endif
