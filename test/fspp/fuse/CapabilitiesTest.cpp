#include <gtest/gtest.h>

#include "fspp/fuse/Capabilities.h"
#include "fspp/fuse/params.h"

using fspp::fuse::removeUnsupportedCapabilities;
using fspp::fuse::unsupportedCapabilities;

namespace {
// What libfuse offers us in the worst case: it wants everything the kernel can do.
constexpr uint32_t ALL_CAPABILITIES = 0xFFFFFFFFu;

bool wants(uint32_t capability) {
  return 0 != (removeUnsupportedCapabilities(ALL_CAPABILITIES) & capability);
}
}

// CryFile::open() ignores the open flags, so it never truncates on O_TRUNC and the kernel has to
// keep sending us a separate truncate request.
TEST(CapabilitiesTest, AtomicOTruncIsRefused) {
  EXPECT_FALSE(wants(FUSE_CAP_ATOMIC_O_TRUNC));
}

// We don't implement locking.
TEST(CapabilitiesTest, PosixLocksAreRefused) {
  EXPECT_FALSE(wants(FUSE_CAP_POSIX_LOCKS));
}

// libfuse 2 had no such capability and it costs us an extra getattr per read, see Capabilities.cpp.
TEST(CapabilitiesTest, AutoInvalDataIsRefused) {
  EXPECT_FALSE(wants(FUSE_CAP_AUTO_INVAL_DATA));
}

// Our readdir() doesn't fill in plus data, so readdirplus replies would be larger for nothing, and
// on libfuse 3.0 to 3.10.2 they lose d_type entirely.
TEST(CapabilitiesTest, ReaddirplusIsRefused) {
  EXPECT_FALSE(wants(FUSE_CAP_READDIRPLUS));
  EXPECT_FALSE(wants(FUSE_CAP_READDIRPLUS_AUTO));
}

// Accepting this would make us responsible for clearing setuid/setgid ourselves, and we aren't.
TEST(CapabilitiesTest, HandleKillprivIsRefused) {
  EXPECT_FALSE(wants(FUSE_CAP_HANDLE_KILLPRIV));
}

// libfuse's own high level library implements this and enables it by default, in libfuse 2 as well,
// so refusing it would newly break NFS export.
TEST(CapabilitiesTest, ExportSupportIsKept) {
  EXPECT_TRUE(wants(FUSE_CAP_EXPORT_SUPPORT));
}

// We only ever clear capabilities, we never ask for one libfuse didn't offer.
TEST(CapabilitiesTest, NoCapabilityIsAdded) {
  EXPECT_EQ(0u, removeUnsupportedCapabilities(0));
  EXPECT_EQ(0u, removeUnsupportedCapabilities(unsupportedCapabilities()));
  const uint32_t someWantedCapability = FUSE_CAP_EXPORT_SUPPORT;
  EXPECT_EQ(someWantedCapability, removeUnsupportedCapabilities(someWantedCapability));
}

// Capabilities libfuse never offered stay off, whatever we do.
TEST(CapabilitiesTest, RemovingIsIdempotent) {
  const uint32_t once = removeUnsupportedCapabilities(ALL_CAPABILITIES);
  EXPECT_EQ(once, removeUnsupportedCapabilities(once));
}
