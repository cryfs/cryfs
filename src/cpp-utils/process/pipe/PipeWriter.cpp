#include "PipeWriter.h"
#include <utility>
#include <stdexcept>

using namespace cpputils::process;
using std::string;

PipeWriter::PipeWriter(PipeDescriptor fd): _stream(std::move(fd), "w") {
}

void PipeWriter::write(const string &str) {
    uint64_t len = str.size();
    size_t res = fwrite(&len, sizeof(len), 1, _stream.stream());
    if (res != 1) {
        throw std::runtime_error("Writing message length to pipe failed.");
    }
    res = fwrite(str.c_str(), 1, len, _stream.stream());
    if (res != len) {
        throw std::runtime_error("Writing message to pipe failed.");
    }
    fflush(_stream.stream());
}
