#pragma once
#ifndef CRYFS_TEST_BLOCKSTORE_IMPLEMENTATIONS_CACHING_TESTUTILS_CALLBACKMOCK_H_
#define CRYFS_TEST_BLOCKSTORE_IMPLEMENTATIONS_CACHING_TESTUTILS_CALLBACKMOCK_H_

#include <gmock/gmock.h>

class CallbackMock final {
public:
    MOCK_METHOD2(call, void(int, int));
};

#endif
