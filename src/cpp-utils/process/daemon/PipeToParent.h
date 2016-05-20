#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_DAEMON_PIPETOPARENT_H
#define MESSMER_CPPUTILS_PROCESS_DAEMON_PIPETOPARENT_H

#include "../../macros.h"
#include "../pipe/PipeWriter.h"

namespace cpputils {
namespace process {

    class PipeToParent final {
    public:
        PipeToParent(PipeWriter writer);
        PipeToParent(PipeToParent &&rhs) = default;

        void notifyReady();
        void notifyError(const std::string &message);

    private:
        PipeWriter _writer;

        DISALLOW_COPY_AND_ASSIGN(PipeToParent);
    };

}
}

#endif
