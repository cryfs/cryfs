#pragma once

#ifndef CRYFS_TIMESTAMPUPDATEBEHAVIOR_H
#define CRYFS_TIMESTAMPUPDATEBEHAVIOR_H

namespace cryfs {
namespace fsblobstore {

enum class TimestampUpdateBehavior : uint8_t {
    // currently only relatime supported
    RELATIME,
    NOATIME
};

}
}

#endif
