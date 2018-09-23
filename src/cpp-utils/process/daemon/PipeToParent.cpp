#include "PipeToParent.h"
#include "Messages.h"

using namespace cpputils::process;
using std::string;

PipeToParent::PipeToParent(PipeWriter writer)
        : _writer(std::move(writer)) {
}

void PipeToParent::notifyReady() {
    _writer.send(Messages::READY);
}

void PipeToParent::notifyError(const string &message) {
    _writer.send(Messages::ERROR);
    _writer.send(message);
}
