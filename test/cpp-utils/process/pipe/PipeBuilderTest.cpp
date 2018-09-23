#include <gtest/gtest.h>

#include <cpp-utils/process/pipe/PipeBuilder.h>
#include <thread>

using cpputils::process::PipeBuilder;
using cpputils::process::PipeReader;
using cpputils::process::PipeWriter;
using std::unique_ptr;
using std::make_unique;

class PipeBuilderTest : public ::testing::Test {
public:
};

TEST_F(PipeBuilderTest, get_nothing) {
    PipeBuilder builder;
}

TEST_F(PipeBuilderTest, get_reader) {
    PipeBuilder builder;
    builder.reader();
}

TEST_F(PipeBuilderTest, get_writer) {
    PipeBuilder builder;
    builder.writer();
}

TEST_F(PipeBuilderTest, get_both) {
    PipeBuilder builder;
    builder.reader();
    builder.writer();
}

TEST_F(PipeBuilderTest, close_reader) {
    PipeBuilder builder;
    builder.closeReader();
}

TEST_F(PipeBuilderTest, close_writer) {
    PipeBuilder builder;
    builder.closeWriter();
}

TEST_F(PipeBuilderTest, close_both) {
    PipeBuilder builder;
    builder.closeReader();
    builder.closeWriter();
}

TEST_F(PipeBuilderTest, try_get_two_readers) {
    PipeBuilder builder;
    builder.reader();
    EXPECT_THROW(
        builder.reader(),
        std::logic_error
    );
}

TEST_F(PipeBuilderTest, try_get_two_writers) {
    PipeBuilder builder;
    builder.writer();
    EXPECT_THROW(
        builder.writer(),
        std::logic_error
    );
}

TEST_F(PipeBuilderTest, try_get_reader_after_closing) {
    PipeBuilder builder;
    builder.closeReader();
    EXPECT_THROW(
        builder.reader(),
        std::logic_error
    );
}

TEST_F(PipeBuilderTest, try_get_writer_after_closing) {
    PipeBuilder builder;
    builder.closeWriter();
    EXPECT_THROW(
        builder.writer(),
        std::logic_error
    );
}

TEST_F(PipeBuilderTest, get_reader_after_closing_writer) {
    PipeBuilder builder;
    builder.closeWriter();
    builder.reader();
}

TEST_F(PipeBuilderTest, get_writer_after_closing_reader) {
    PipeBuilder builder;
    builder.closeReader();
    builder.writer();
}
