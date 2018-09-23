#include "PipeFromChild.h"
#include "Messages.h"

using namespace cpputils::process;
using std::string;
using boost::optional;
using boost::none;

PipeFromChild::PipeFromChild(PipeReader reader)
        : _reader(std::move(reader)) {
}

optional<std::string> PipeFromChild::waitForReadyReturnError() {
    string msg;
    try {
        msg = _reader.receive();
    } catch (const PipeNotReadableError &e) {
        return string("Child exited before being ready.");
    }
    if (msg == Messages::READY) {
        return none;
    } else if (msg == Messages::ERROR) {
        string errorMessage = _reader.receive();
        return errorMessage;
    } else {
        throw std::logic_error("Received unknown message");
    }
}
