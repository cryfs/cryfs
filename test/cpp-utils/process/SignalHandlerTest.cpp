#include <gtest/gtest.h>
#include <cpp-utils/process/SignalHandler.h>

using namespace cpputils;

namespace {
std::atomic<int> triggered;

void trigger(int signal) {
    triggered = signal;
}

void raise_signal(int signal) {
    const int error = ::raise(signal);
    if (error != 0) {
        throw std::runtime_error("Error raising signal");
    }
}

TEST(SignalHandlerTest, givenNoSignalHandler_whenRaisingSigint_thenDies) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
    EXPECT_DEATH(
        raise_signal(SIGINT),
        ""
    );
}

TEST(SignalHandlerTest, givenNoSignalHandler_whenRaisingSigterm_thenDies) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
    EXPECT_DEATH(
        raise_signal(SIGTERM),
        ""
    );
}

TEST(SignalHandlerTest, givenSigIntHandler_whenRaisingSigInt_thenCatches) {
    triggered = 0;

    const SignalHandlerRAII<&trigger> handler(SIGINT);

    raise_signal(SIGINT);
    EXPECT_EQ(SIGINT, triggered);
}

TEST(SignalHandlerTest, givenSigIntHandler_whenRaisingSigTerm_thenDies) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
    const SignalHandlerRAII<&trigger> handler(SIGINT);

    EXPECT_DEATH(
        raise_signal(SIGTERM),
        ""
    );
}

TEST(SignalHandlerTest, givenSigTermHandler_whenRaisingSigTerm_thenCatches) {
    triggered = 0;

    const SignalHandlerRAII<&trigger> handler(SIGTERM);

    raise_signal(SIGTERM);
    EXPECT_EQ(SIGTERM, triggered);
}

TEST(SignalHandlerTest, givenSigTermHandler_whenRaisingSigInt_thenDies) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
    const SignalHandlerRAII<&trigger> handler(SIGTERM);

    EXPECT_DEATH(
        raise_signal(SIGINT),
        ""
    );
}

TEST(SignalHandlerTest, givenSigIntAndSigTermHandlers_whenRaising_thenCatchesCorrectSignal) {
    triggered = 0;

    const SignalHandlerRAII<&trigger> handler1(SIGINT);
    const SignalHandlerRAII<&trigger> handler2(SIGTERM);

    raise_signal(SIGINT);
    EXPECT_EQ(SIGINT, triggered);

    raise_signal(SIGTERM);
    EXPECT_EQ(SIGTERM, triggered);

    raise_signal(SIGINT);
    EXPECT_EQ(SIGINT, triggered);
}

std::atomic<int> triggered_count_1;
std::atomic<int> triggered_count_2;

void trigger1(int) {
    ++triggered_count_1;
}

void trigger2(int) {
    ++triggered_count_2;
}

TEST(SignalHandlerTest, givenMultipleSigIntHandlers_whenRaising_thenCatchesCorrectSignal) {
    testing::FLAGS_gtest_death_test_style = "threadsafe";
    triggered_count_1 = 0;
    triggered_count_2 = 0;

    {
        const SignalHandlerRAII<&trigger1> handler1(SIGINT);

        {
            const SignalHandlerRAII<&trigger2> handler2(SIGINT);

            raise_signal(SIGINT);
            EXPECT_EQ(0, triggered_count_1);
            EXPECT_EQ(1, triggered_count_2);

            raise_signal(SIGINT);
            EXPECT_EQ(0, triggered_count_1);
            EXPECT_EQ(2, triggered_count_2);
        }

        raise_signal(SIGINT);
        EXPECT_EQ(1, triggered_count_1);
        EXPECT_EQ(2, triggered_count_2);

        raise_signal(SIGINT);
        EXPECT_EQ(2, triggered_count_1);
        EXPECT_EQ(2, triggered_count_2);

    }

    EXPECT_DEATH(
        raise_signal(SIGINT),
        ""
    );
}

}
