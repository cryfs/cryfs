#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_CONTEXT_H_
#define MESSMER_FSPP_FSINTERFACE_CONTEXT_H_

#include <cstdint>
#include <stdexcept>
#include <memory>
#include <cpp-utils/system/time.h>

namespace fspp {

namespace detail {
enum class TimestampUpdateBehaviorBase {
    NOATIME,
    STRICTATIME,
    RELATIME,
    NODIRATIME_STRICTATIME,
    NODIRATIME_RELATIME,
};
}


// Defines how atime timestamps of files and directories are accessed on read accesses
// (e.g. atime, strictatime, relatime, nodiratime)
using TimestampUpdateBehavior = std::shared_ptr<detail::TimestampUpdateBehaviorBase>;

inline bool _relatime(timespec oldATime, timespec oldMTime, timespec newATime) {
    const timespec yesterday {
            /*.tv_sec = */ newATime.tv_sec - 60*60*24,
            /*.tv_nsec = */ newATime.tv_nsec
    };

    return oldATime < oldMTime || oldATime < yesterday;
}

inline bool shouldUpdateATimeOnFileRead(const TimestampUpdateBehavior &behavior, timespec oldATime, timespec oldMTime, timespec newATime) {
    switch (*behavior) {
        case detail::TimestampUpdateBehaviorBase::NOATIME:
            return false;
        case detail::TimestampUpdateBehaviorBase::STRICTATIME:
        case detail::TimestampUpdateBehaviorBase::NODIRATIME_STRICTATIME:
            return true;
        case detail::TimestampUpdateBehaviorBase::RELATIME:
        case detail::TimestampUpdateBehaviorBase::NODIRATIME_RELATIME:
            return _relatime(oldATime, oldMTime, newATime);
        default:
            throw std::runtime_error("Unknown TimestampUpdateBehavior");
    }
}

inline bool shouldUpdateAtimeOnDirectoryRead(const TimestampUpdateBehavior &behavior, timespec oldATime, timespec oldMTime, timespec newATime) {
    switch (*behavior) {
        case detail::TimestampUpdateBehaviorBase::NOATIME:
        case detail::TimestampUpdateBehaviorBase::NODIRATIME_RELATIME:
        case detail::TimestampUpdateBehaviorBase::NODIRATIME_STRICTATIME:
            return false;
        case detail::TimestampUpdateBehaviorBase::STRICTATIME:
            return true;
        case detail::TimestampUpdateBehaviorBase::RELATIME:
            return _relatime(oldATime, oldMTime, newATime);
        default:
            throw std::runtime_error("Unknown TimestampUpdateBehavior");
    }
}

// atime attribute (of both files and directories) is updated only during write access.
inline TimestampUpdateBehavior noatime() {
    static std::shared_ptr<detail::TimestampUpdateBehaviorBase> singleton =
        std::make_shared<detail::TimestampUpdateBehaviorBase>(detail::TimestampUpdateBehaviorBase::NOATIME);
    return singleton;
}

// This causes the atime attribute to update with every file access. (accessing file data, not just the metadata/attributes)
inline TimestampUpdateBehavior strictatime() {
    static std::shared_ptr<detail::TimestampUpdateBehaviorBase> singleton =
        std::make_shared<detail::TimestampUpdateBehaviorBase>(detail::TimestampUpdateBehaviorBase::STRICTATIME);
    return singleton;
}

// This option causes the atime attribute to update only if the previous atime is older than mtime or ctime, or the previous atime is over 24 hours old.
inline TimestampUpdateBehavior relatime() {
    static std::shared_ptr<detail::TimestampUpdateBehaviorBase> singleton =
        std::make_shared<detail::TimestampUpdateBehaviorBase>(detail::TimestampUpdateBehaviorBase::RELATIME);
    return singleton;
}

// atime of directories is updated only during write access, can be combined with relatime. atime of files follows the relatime rules.
inline TimestampUpdateBehavior nodiratime_relatime() {
    static std::shared_ptr<detail::TimestampUpdateBehaviorBase> singleton =
        std::make_shared<detail::TimestampUpdateBehaviorBase>(detail::TimestampUpdateBehaviorBase::NODIRATIME_RELATIME);
    return singleton;
}

// atime of directories is updated only during write access, can be combined with relatime. atime of files follows the strictatime rules.
inline TimestampUpdateBehavior nodiratime_strictatime() {
    static std::shared_ptr<detail::TimestampUpdateBehaviorBase> singleton =
        std::make_shared<detail::TimestampUpdateBehaviorBase>(detail::TimestampUpdateBehaviorBase::NODIRATIME_STRICTATIME);
    return singleton;
}

class Context final {
public:
    explicit Context(TimestampUpdateBehavior timestampUpdateBehavior)
    : _timestampUpdateBehavior(std::move(timestampUpdateBehavior)) {}

    const TimestampUpdateBehavior& timestampUpdateBehavior() const {
        return _timestampUpdateBehavior;
    }

    void setTimestampUpdateBehavior(TimestampUpdateBehavior value) {
        _timestampUpdateBehavior = std::move(value);
    }

private:
    TimestampUpdateBehavior _timestampUpdateBehavior;
};

}

#endif
