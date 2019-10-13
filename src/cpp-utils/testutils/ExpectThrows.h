#pragma once

#ifndef MESSMER_CPPUTILS_EXPECTTHROWS_H
#define MESSMER_CPPUTILS_EXPECTTHROWS_H

#include <gmock/gmock.h>
#include <cpp-utils/assert/assert.h>

namespace cpputils {

template<class Exception, class Functor>
inline void expectThrows(Functor&& functor, const char* expectMessageContains) {
    try {
        std::forward<Functor>(functor)();
    } catch (const Exception& e) {
        EXPECT_THAT(e.what(), testing::HasSubstr(expectMessageContains));
        return;
    }
    ADD_FAILURE() << "Expected to throw exception containing \""
                  << expectMessageContains << "\" but didn't throw";
}

template<class Functor>
inline void expectFailsAssertion(Functor&& functor, const char* expectMessageContains) {
    cpputils::_assert::DisableAbortOnFailedAssertionRAII _disableAbortOnFailedAssertionRAII;
    expectThrows<cpputils::AssertFailed>(std::forward<Functor>(functor), expectMessageContains);
}

}

#endif
