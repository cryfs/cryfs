#include "PipeStreamEndpoint.h"
#include "../../logging/logging.h"
#include "../../assert/assert.h"

using namespace cpputils::process;
using namespace cpputils::logging;

PipeStreamEndpoint::PipeStreamEndpoint(PipeDescriptor fd, const char *mode): _fd(std::move(fd)), _stream(0) {
    ASSERT(_fd.valid(), "Given PipeDescriptor not valid");
    _stream = fdopen(_fd.fd(), mode);
    if (nullptr == _stream) {
        throw std::runtime_error("Failed to fdopen() pipe. errno: "+std::to_string(errno));
    }
}

PipeStreamEndpoint::PipeStreamEndpoint(PipeStreamEndpoint &&rhs): _fd(std::move(rhs._fd)), _stream(rhs._stream) {
    rhs._stream = nullptr;
}

PipeStreamEndpoint::~PipeStreamEndpoint() {
    ASSERT(_fd.valid() == (_stream != nullptr), "Either both, _fd and _stream, should be valid or invalid");
    if (nullptr != _stream) {
        if (0 != fclose(_stream)) {
            LOG(ERR, "Failed to fclose() pipe. errno: {}", errno);
        }
        _fd.wasClosedOutside();
        _stream = nullptr;
    }
}

FILE *PipeStreamEndpoint::stream() {
    ASSERT(nullptr != _stream, "PipeStreamEndpoint invalid");
    return _stream;
}
