#pragma once
#ifndef MESSMER_MYGTESTMAIN_MOUNTAVAILABILITY_H
#define MESSMER_MYGTESTMAIN_MOUNTAVAILABILITY_H

#include <cstdlib>
#include <gtest/gtest.h>

// Not every machine the tests run on can mount a file system. GitHub's hosted macOS runners for
// example can't load the macFUSE kernel extension, which needs a user to approve it and to reboot.
// CI sets CRYFS_TEST_CANNOT_MOUNT there (see .github/workflows/actions/run_tests/action.yaml), and
// the tests that need a mounted file system skip instead of failing, so everything else in their
// test binaries still runs.
inline bool mounting_is_unavailable() {
    const char* value = std::getenv("CRYFS_TEST_CANNOT_MOUNT");
    return value != nullptr && value[0] != '\0';
}

// Skips the current test, or the whole suite when used from SetUpTestSuite(), where mounting isn't
// possible.
#define SKIP_IF_MOUNTING_IS_UNAVAILABLE()                                                            \
    do {                                                                                              \
        if (mounting_is_unavailable()) {                                                              \
            GTEST_SKIP() << "This test needs to mount a file system, which isn't possible here "     \
                            "(CRYFS_TEST_CANNOT_MOUNT is set)";                                       \
        }                                                                                             \
    } while (0)

#endif
