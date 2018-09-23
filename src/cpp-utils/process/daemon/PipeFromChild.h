#pragma once
#ifndef MESSMER_CPPUTILS_PROCESS_DAEMON_PIPEFROMCHILD_H
#define MESSMER_CPPUTILS_PROCESS_DAEMON_PIPEFROMCHILD_H

#include <cpp-utils/macros.h>
#include <boost/optional.hpp>
#include "../pipe/PipeReader.h"

namespace cpputils {
namespace process {

    class PipeFromChild final {
    public:
        PipeFromChild(PipeReader reader);
        PipeFromChild(PipeFromChild &&rhs) = default;

        // TODO Give this a timeout parameter
        boost::optional<std::string> waitForReadyReturnError();
    private:
        PipeReader _reader;

        DISALLOW_COPY_AND_ASSIGN(PipeFromChild);
    };

}
}


#endif
