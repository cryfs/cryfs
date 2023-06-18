#include <cpp-utils/process/SignalCatcher.h>
#include <gtest/gtest.h>
#include <csignal>

using cpputils::SignalCatcher;

namespace {
void raise_signal(int signal) {
    int error = ::raise(signal);
    if (error != 0) {
        throw std::runtime_error("Error raising signal");
    }
}
}

TEST(SignalCatcherTest, givenNoSignalCatcher_whenRaisingSigint_thenDies) {
    EXPECT_DEATH(
        raise_signal(SIGINT),
        ""
    );
}

TEST(SignalCatcherTest, givenNoSignalCatcher_whenRaisingSigterm_thenDies) {
    EXPECT_DEATH(
        raise_signal(SIGTERM),
        ""
    );
}

TEST(SignalCatcherTest, givenSigIntCatcher_whenRaisingSigInt_thenCatches) {
    SignalCatcher catcher({SIGINT});

    EXPECT_FALSE(catcher.signal_occurred());
    raise_signal(SIGINT);
    EXPECT_TRUE(catcher.signal_occurred());

    // raise again
    raise_signal(SIGINT);
    EXPECT_TRUE(catcher.signal_occurred());
}

TEST(SignalCatcherTest, givenSigTermCatcher_whenRaisingSigTerm_thenCatches) {
    SignalCatcher catcher({SIGTERM});

    EXPECT_FALSE(catcher.signal_occurred());
    raise_signal(SIGTERM);
    EXPECT_TRUE(catcher.signal_occurred());

    // raise again
    raise_signal(SIGTERM);
    EXPECT_TRUE(catcher.signal_occurred());
}

TEST(SignalCatcherTest, givenSigIntAndSigTermCatcher_whenRaisingSigInt_thenCatches) {
    SignalCatcher catcher({SIGINT, SIGTERM});

    EXPECT_FALSE(catcher.signal_occurred());
    raise_signal(SIGINT);
    EXPECT_TRUE(catcher.signal_occurred());

    // raise again
    raise_signal(SIGINT);
    EXPECT_TRUE(catcher.signal_occurred());
}

TEST(SignalCatcherTest, givenSigIntAndSigTermCatcher_whenRaisingSigTerm_thenCatches) {
    SignalCatcher catcher({SIGINT, SIGTERM});

    EXPECT_FALSE(catcher.signal_occurred());
    raise_signal(SIGTERM);
    EXPECT_TRUE(catcher.signal_occurred());

    // raise again
    raise_signal(SIGTERM);
    EXPECT_TRUE(catcher.signal_occurred());
}

TEST(SignalCatcherTest, givenSigIntAndSigTermCatcher_whenRaisingSigIntAndSigTerm_thenCatches) {
    SignalCatcher catcher({SIGINT, SIGTERM});

    EXPECT_FALSE(catcher.signal_occurred());
    raise_signal(SIGTERM);
    EXPECT_TRUE(catcher.signal_occurred());

    raise_signal(SIGINT);
    EXPECT_TRUE(catcher.signal_occurred());
}

TEST(SignalCatcherTest, givenSigIntCatcherAndSigTermCatcher_whenRaisingSignalsInOrder_thenCorrectCatcherCatches) {
    SignalCatcher sigintCatcher({SIGINT});
    SignalCatcher sigtermCatcher({SIGTERM});

    EXPECT_FALSE(sigintCatcher.signal_occurred());
    raise_signal(SIGINT);
    EXPECT_TRUE(sigintCatcher.signal_occurred());

    EXPECT_FALSE(sigtermCatcher.signal_occurred());
    raise_signal(SIGTERM);
    EXPECT_TRUE(sigtermCatcher.signal_occurred());
}

TEST(SignalCatcherTest, givenSigIntCatcherAndSigTermCatcher_whenRaisingSignalsInReverseOrder_thenCorrectCatcherCatches) {
    SignalCatcher sigintCatcher({SIGINT});
    SignalCatcher sigtermCatcher({SIGTERM});

    EXPECT_FALSE(sigtermCatcher.signal_occurred());
    raise_signal(SIGTERM);
    EXPECT_TRUE(sigtermCatcher.signal_occurred());

    EXPECT_FALSE(sigintCatcher.signal_occurred());
    raise_signal(SIGINT);
    EXPECT_TRUE(sigintCatcher.signal_occurred());
}

TEST(SignalCatcherTest, givenNestedSigIntCatchers_whenRaisingSignals_thenCorrectCatcherCatches) {
    SignalCatcher outerCatcher({SIGINT});
    {
        SignalCatcher middleCatcher({SIGINT});

        EXPECT_FALSE(middleCatcher.signal_occurred());
        raise_signal(SIGINT);
        EXPECT_TRUE(middleCatcher.signal_occurred());

        {
            SignalCatcher innerCatcher({SIGINT});

            EXPECT_FALSE(innerCatcher.signal_occurred());
            raise_signal(SIGINT);
            EXPECT_TRUE(innerCatcher.signal_occurred());
        }
    }

    EXPECT_FALSE(outerCatcher.signal_occurred());
    raise_signal(SIGINT);
    EXPECT_TRUE(outerCatcher.signal_occurred());
}

TEST(SignalCatcherTest, givenExpiredSigIntCatcher_whenRaisingSigInt_thenDies) {
    {
        SignalCatcher catcher({SIGINT});
    }

    EXPECT_DEATH(
        raise_signal(SIGINT),
        ""
    );
}

TEST(SignalCatcherTest, givenExpiredSigTermCatcher_whenRaisingSigTerm_thenDies) {
    {
        SignalCatcher catcher({SIGTERM});
    }

    EXPECT_DEATH(
        raise_signal(SIGTERM),
        ""
    );
}

TEST(SignalCatcherTest, givenExpiredSigIntCatcherAndSigTermCatcher_whenRaisingSigTerm_thenDies) {
    {
        SignalCatcher sigIntCatcher({SIGTERM});
        SignalCatcher sigTermCatcer({SIGTERM});
    }

    EXPECT_DEATH(
        raise_signal(SIGTERM),
        ""
    );
}

TEST(SignalCatcherTest, givenSigTermCatcherAndExpiredSigIntCatcher_whenRaisingSigTerm_thenCatches) {
    SignalCatcher sigTermCatcher({SIGTERM});
    {
        SignalCatcher sigIntCatcher({SIGINT});
    }

    EXPECT_FALSE(sigTermCatcher.signal_occurred());
    raise_signal(SIGTERM);
    EXPECT_TRUE(sigTermCatcher.signal_occurred());
}

TEST(SignalCatcherTest, givenSigTermCatcherAndExpiredSigIntCatcher_whenRaisingSigInt_thenDies) {
    SignalCatcher sigTermCacher({SIGTERM});
    {
        SignalCatcher sigIntCatcher({SIGINT});
    }

    EXPECT_DEATH(
        raise_signal(SIGINT),
        ""
    );
}
