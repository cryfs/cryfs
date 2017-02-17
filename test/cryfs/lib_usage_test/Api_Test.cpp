#include "testutils/C_Library_Test.h"

class Api_Test : public ::testing::Test {
public:
  static constexpr const int API_VERSION = 1;
};

TEST_F(Api_Test, init_and_free) {
  cryfs_api_context *api = nullptr;
  EXPECT_SUCCESS(cryfs_init(API_VERSION, &api));
  EXPECT_NE(nullptr, api);
  cryfs_free(&api);
  EXPECT_EQ(nullptr, api);
}

TEST_F(Api_Test, init_unsupported_api_version) {
  cryfs_api_context *api = (cryfs_api_context*)0x4; // Initialise to something else than nullptr
  EXPECT_EQ(cryfs_error_UNSUPPORTED_API_VERSION, cryfs_init(2, &api));
  EXPECT_EQ(nullptr, api);
}

TEST_F(Api_Test, free_with_nullptr_doesnt_crash_1) {
  cryfs_free(nullptr);
}

TEST_F(Api_Test, free_with_nullptr_doesnt_crash_2) {
  cryfs_api_context *context = nullptr;
  cryfs_free(&context);
}

TEST_F(Api_Test, loadcontext_init_and_free_globally) {
  cryfs_api_context *api;
  cryfs_load_context *context;
  EXPECT_SUCCESS(cryfs_init(API_VERSION, &api));
  EXPECT_SUCCESS(cryfs_load_init(api, &context));
  // Don't free the load context, it is freed in the cryfs_free(api) call of the test fixture
  // This test can be helpful if run with valgrind to check that this isn't a memory leak
  cryfs_free(&api);
}

TEST_F(Api_Test, createcontext_init_and_free_globally) {
  cryfs_api_context *api;
  cryfs_create_context *context;
  EXPECT_SUCCESS(cryfs_init(API_VERSION, &api));
  EXPECT_SUCCESS(cryfs_create_init(api, &context));
  // Don't free the create context, it is freed in the cryfs_free(api) call of the test fixture
  // This test can be helpful if run with valgrind to check that this isn't a memory leak
  cryfs_free(&api);
}

TEST_F(Api_Test, free_twice) {
  cryfs_api_context *api;
  EXPECT_SUCCESS(cryfs_init(API_VERSION, &api));
  cryfs_free(&api);
  cryfs_free(&api);
}
