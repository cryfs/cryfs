#include "testutils/C_Library_Test.h"

class Api_Test : public C_Library_Test {
};

TEST_F(Api_Test, init_and_free) {
  cryfs_api_context *context;
  EXPECT_SUCCESS(cryfs_init(API_VERSION, &context));
  cryfs_free(context);
}

TEST_F(Api_Test, init_unsupported_api_version) {
  cryfs_api_context *context = (cryfs_api_context*)0x4; // Initialise to something else than nullptr
  EXPECT_EQ(cryfs_error_UNSUPPORTED_API_VERSION, cryfs_init(2, &context));
  EXPECT_EQ(nullptr, context);
  cryfs_free(context); // Test that people can call cryfs_load_free after an error in cryfs_load_init
}
/*
 * // TODO ...
Test(Api_Test, loadcontext_init_and_free) {
  cryfs_api_context *context;
  EXPECT_SU
}*/
