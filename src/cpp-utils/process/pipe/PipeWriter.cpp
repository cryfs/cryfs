#include "PipeWriter.h"
#include <utility>
#include <stdexcept>
#include "PipeReader.h"

using namespace cpputils::process;
using std::string;

PipeWriter::PipeWriter(PipeDescriptor fd): _stream(std::move(fd), "w") {
}

void PipeWriter::send(const string &str) {
    uint64_t len = str.size();

    // PipeReader will reject messages larger than MAX_READ_SIZE to protect against memory attacks
    if (len > PipeReader::MAX_READ_SIZE) {
        throw std::runtime_error("Message too large.");
    }

    size_t res = fwrite(&len, sizeof(len), 1, _stream.stream());
    if (res != 1) {
        throw std::runtime_error("Writing message length to pipe failed.");
    }
    if (len > 0) {
        res = fwrite(str.c_str(), len, 1, _stream.stream());
        if (res != 1) {
            throw std::runtime_error("Writing message to pipe failed.");
        }
    }
    fflush(_stream.stream());
}
