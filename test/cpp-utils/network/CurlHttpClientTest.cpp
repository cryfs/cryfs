#include <gtest/gtest.h>
#include <gmock/gmock.h>
#include "cpp-utils/network/CurlHttpClient.h"
#include "cpp-utils/pointer/unique_ref_boost_optional_gtest_workaround.h"

using std::string;
using boost::none;
using testing::MatchesRegex;

using namespace cpputils;

TEST(CurlHttpClientTest, InvalidProtocol) {
    EXPECT_EQ(none, CurlHttpClient().get("invalid://example.com"));
}

TEST(CurlHttpClientTest, InvalidTld) {
    EXPECT_EQ(none, CurlHttpClient().get("http://example.invalidtld"));
}

TEST(CurlHttpClientTest, InvalidDomain) {
    EXPECT_EQ(none, CurlHttpClient().get("http://this_is_a_not_existing_domain.com"));
}

TEST(CurlHttpClientTest, ValidHttp) {
    string content = CurlHttpClient().get("http://example.com").value();
    EXPECT_THAT(content, MatchesRegex(".*Example Domain.*"));
}

TEST(CurlHttpClientTest, ValidHttps) {
    string content = CurlHttpClient().get("https://example.com").value();
    EXPECT_THAT(content, MatchesRegex(".*Example Domain.*"));
}
