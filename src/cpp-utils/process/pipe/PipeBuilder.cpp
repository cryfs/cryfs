#include "PipeBuilder.h"
#include "../../logging/logging.h"
#include "../../assert/assert.h"
#include <unistd.h>

using namespace cpputils::process;
using namespace cpputils::logging;

PipeBuilder::PipeBuilder(): _readFd(), _writeFd() {
    int fds[2];
    if (0 != pipe(fds)) {
        throw std::runtime_error("pipe() syscall failed");
    }
    _readFd = PipeDescriptor(fds[0]);
    _writeFd = PipeDescriptor(fds[1]);
}

PipeReader PipeBuilder::reader() {
    if (!_readFd.valid()) {
        throw std::logic_error("Reader was already requested before or closed.");
    }

    // Return read end
    return PipeReader(std::move(_readFd));
}

PipeWriter PipeBuilder::writer() {
    if (!_writeFd.valid()) {
        throw std::logic_error("Writer was already requested before or closed.");
    }

    // Return write end
    return PipeWriter(std::move(_writeFd));
}

void PipeBuilder::closeReader() {
    _readFd.close();
}

void PipeBuilder::closeWriter() {
    _writeFd.close();
}
