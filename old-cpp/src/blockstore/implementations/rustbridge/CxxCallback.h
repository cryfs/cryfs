#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_RUSTBRIDGE_CXXCALLBACK_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_RUSTBRIDGE_CXXCALLBACK_H_

#include <functional>

namespace blockstore
{
    namespace rust
    {
        class CxxCallback
        {
        public:
            explicit CxxCallback(std::function<void ()> callback): _callback(std::move(callback)) {}

            void call() const {
                _callback();
            }
        private:
            std::function<void ()> _callback;
        };
    }
}

#endif
