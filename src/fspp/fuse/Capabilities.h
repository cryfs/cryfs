#pragma once
#ifndef MESSMER_FSPP_FUSE_CAPABILITIES_H_
#define MESSMER_FSPP_FUSE_CAPABILITIES_H_

#include <cstdint>

namespace fspp {
namespace fuse {

/**
 * Before libfuse calls our init() handler, it fills fuse_conn_info::want with the capabilities it
 * would like to negotiate with the kernel. Some of those we can't support and some we don't want,
 * so this takes the set libfuse proposed and returns the set we actually want.
 *
 * The implementation documents why each capability is in or out.
 */
uint32_t removeUnsupportedCapabilities(uint32_t want);

/**
 * The capabilities removeUnsupportedCapabilities() clears. Exposed for tests.
 */
uint32_t unsupportedCapabilities();

}
}

#endif
