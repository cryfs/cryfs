#include "PipeDescriptor.h"
#include "../../logging/logging.h"
#include "../../assert/assert.h"
#include <unistd.h>

using namespace cpputils::process;
using namespace cpputils::logging;

PipeDescriptor::PipeDescriptor(): _fd(-1) {
}

PipeDescriptor::PipeDescriptor(int fd): _fd(fd) {
}

PipeDescriptor::PipeDescriptor(PipeDescriptor &&rhs): _fd(rhs._fd) {
    rhs._fd = -1;
}

PipeDescriptor &PipeDescriptor::operator=(PipeDescriptor &&rhs) {
    ASSERT(&rhs != this, "Can't move-assign to this");

    _destruct();
    _fd = rhs._fd;
    rhs._fd = -1;
    return *this;
}

PipeDescriptor::~PipeDescriptor() {
    try {
        _destruct();
    } catch(const std::exception &e) {
        // Destructors shouldn't throw
        LOG(ERROR) << e.what();
    }
}

void PipeDescriptor::_destruct() {
    if (valid()) {
        close();
    }
}

bool PipeDescriptor::valid() const {
    return _fd != -1;
}

int PipeDescriptor::fd() {
    ASSERT(valid(), "PipeDescriptor invalid");
    return _fd;
}

void PipeDescriptor::close() {
    if (-1 == _fd) {
        throw std::logic_error("Pipe already closed");
    }
    if (0 != ::close(_fd)) {
        _fd = -1;
        throw std::runtime_error("Error closing pipe with close() syscall. errno: " + std::to_string(errno));
    }
    _fd = -1;
}

void PipeDescriptor::wasClosedOutside() {
    _fd = -1;
}