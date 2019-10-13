#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_CONTEXT_H_
#define MESSMER_FSPP_FSINTERFACE_CONTEXT_H_

#include <cstdint>
#include <memory>
#include <cpp-utils/system/time.h>

namespace fspp {

namespace detail {
class TimestampUpdateBehaviorBase {
public:
    virtual bool shouldUpdateATimeOnFileRead(timespec oldATime, timespec oldMTime, timespec newATime) const = 0;
    virtual bool shouldUpdateATimeOnDirectoryRead(timespec oldATime, timespec oldMTime, timespec newATime) const = 0;
};
}

// Defines how atime timestamps of files and directories are accessed on read accesses
// (e.g. atime, strictatime, relatime, nodiratime)
using TimestampUpdateBehavior = std::shared_ptr<detail::TimestampUpdateBehaviorBase>;

// atime attribute (of both files and directories) is updated only during write access.
inline TimestampUpdateBehavior noatime() {
    class BehaviorImpl final : public detail::TimestampUpdateBehaviorBase {
    public:
        bool shouldUpdateATimeOnFileRead(timespec /*oldATime*/, timespec /*oldMTime*/, timespec /*newATime*/) const override {
            return false;
        }
        bool shouldUpdateATimeOnDirectoryRead(timespec /*oldATime*/, timespec /*oldMTime*/, timespec /*newATime*/) const override {
            return false;
        }
    };

    static std::shared_ptr<BehaviorImpl> singleton = std::make_shared<BehaviorImpl>();
    return singleton;
}

// This causes the atime attribute to update with every file access. (accessing file data, not just the metadata/attributes)
inline TimestampUpdateBehavior strictatime() {
    class BehaviorImpl final : public detail::TimestampUpdateBehaviorBase {
    public:
        bool shouldUpdateATimeOnFileRead(timespec /*oldATime*/, timespec /*oldMTime*/, timespec /*newATime*/) const override {
            return true;
        }
        bool shouldUpdateATimeOnDirectoryRead(timespec /*oldATime*/, timespec /*oldMTime*/, timespec /*newATime*/) const override {
            return true;
        }
    };

    static std::shared_ptr<BehaviorImpl> singleton = std::make_shared<BehaviorImpl>();
    return singleton;
}

// This option causes the atime attribute to update only if the previous atime is older than mtime or ctime, or the previous atime is over 24 hours old.
inline TimestampUpdateBehavior relatime() {
    // This option causes the atime attribute to update only if the previous atime is older than mtime or ctime, or the previous atime is over 24 hours old.
    class BehaviorImpl final : public detail::TimestampUpdateBehaviorBase {
    public:
        bool shouldUpdateATimeOnFileRead(timespec oldATime, timespec oldMTime, timespec newATime) const override {
            const timespec yesterday {
                    /*.tv_sec = */ newATime.tv_sec - 60*60*24,
                    /*.tv_nsec = */ newATime.tv_nsec
            };

            return oldATime < oldMTime || oldATime < yesterday;
        }
        bool shouldUpdateATimeOnDirectoryRead(timespec oldATime, timespec oldMTime, timespec newATime) const override {
            return shouldUpdateATimeOnFileRead(oldATime, oldMTime, newATime);
        }
    };

    static std::shared_ptr<BehaviorImpl> singleton = std::make_shared<BehaviorImpl>();
    return singleton;
}

// atime of directories is updated only during write access, can be combined with relatime. atime of files follows the relatime rules.
inline TimestampUpdateBehavior nodiratime_relatime() {
    class BehaviorImpl final : public detail::TimestampUpdateBehaviorBase {
    public:
        bool shouldUpdateATimeOnFileRead(timespec oldATime, timespec oldMTime, timespec newATime) const override {
            return relatime()->shouldUpdateATimeOnFileRead(oldATime, oldMTime, newATime);
        }
        bool shouldUpdateATimeOnDirectoryRead(timespec /*oldATime*/, timespec /*oldMTime*/, timespec /*newATime*/) const override {
            return false;
        }
    };

    static std::shared_ptr<BehaviorImpl> singleton = std::make_shared<BehaviorImpl>();
    return singleton;
}

// atime of directories is updated only during write access, can be combined with relatime. atime of files follows the strictatime rules.
inline TimestampUpdateBehavior nodiratime_strictatime() {
    class BehaviorImpl final : public detail::TimestampUpdateBehaviorBase {
    public:
        bool shouldUpdateATimeOnFileRead(timespec oldATime, timespec oldMTime, timespec newATime) const override {
            return strictatime()->shouldUpdateATimeOnFileRead(oldATime, oldMTime, newATime);
        }
        bool shouldUpdateATimeOnDirectoryRead(timespec /*oldATime*/, timespec /*oldMTime*/, timespec /*newATime*/) const override {
            return false;
        }
    };

    static std::shared_ptr<BehaviorImpl> singleton = std::make_shared<BehaviorImpl>();
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
