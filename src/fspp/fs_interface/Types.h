#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_TYPES_H_
#define MESSMER_FSPP_FSINTERFACE_TYPES_H_

#include <cstdint>
#include <ctime>
#include <cpp-utils/value_type/ValueType.h>

namespace fspp {

struct uid_t final : cpputils::value_type::IdValueType<uid_t, uint32_t> {
    // TODO Remove default constructor
    constexpr uid_t() noexcept: IdValueType(0) {}

    constexpr explicit uid_t(uint32_t id) noexcept: IdValueType(id) {}

    constexpr uint32_t value() const noexcept {
        return value_;
    }
};

struct gid_t final : cpputils::value_type::IdValueType<gid_t, uint32_t> {
    // TODO Remove default constructor
    constexpr gid_t() noexcept: IdValueType(0) {}

    constexpr explicit gid_t(uint32_t id) noexcept: IdValueType(id) {}

    constexpr uint32_t value() const noexcept {
        return value_;
    }
};

struct mode_t final : cpputils::value_type::FlagsValueType<mode_t, uint32_t> {
    // TODO Remove default constructor
    constexpr mode_t() noexcept: FlagsValueType(0) {}

    constexpr explicit mode_t(uint32_t id) noexcept: FlagsValueType(id) {}

    constexpr uint32_t value() const noexcept {
        return value_;
    }

    constexpr mode_t& addFileFlag() noexcept {
        value_ |= S_IFREG;
        return *this;
    }

    constexpr mode_t& addDirFlag() noexcept {
        value_ |= S_IFDIR;
        return *this;
    }

    constexpr mode_t& addSymlinkFlag() noexcept {
        value_ |= S_IFLNK;
        return *this;
    }

    constexpr mode_t& addUserReadFlag() noexcept {
        value_ |= S_IRUSR;
        return *this;
    }

    constexpr mode_t& addUserWriteFlag() noexcept {
        value_ |= S_IWUSR;
        return *this;
    }

    constexpr mode_t& addUserExecFlag() noexcept {
        value_ |= S_IXUSR;
        return *this;
    }

    constexpr mode_t& addGroupReadFlag() noexcept {
        value_ |= S_IRGRP;
        return *this;
    }

    constexpr mode_t& addGroupWriteFlag() noexcept {
        value_ |= S_IWGRP;
        return *this;
    }

    constexpr mode_t& addGroupExecFlag() noexcept {
        value_ |= S_IXGRP;
        return *this;
    }

    constexpr mode_t& addOtherReadFlag() noexcept {
        value_ |= S_IROTH;
        return *this;
    }

    constexpr mode_t& addOtherWriteFlag() noexcept {
        value_ |= S_IWOTH;
        return *this;
    }

    constexpr mode_t& addOtherExecFlag() noexcept {
        value_ |= S_IXOTH;
        return *this;
    }

    constexpr bool hasFileFlag() const noexcept {
        return S_ISREG(value_);
    }

    constexpr bool hasDirFlag() const noexcept {
        return S_ISDIR(value_);
    }

    constexpr bool hasSymlinkFlag() const noexcept {
        return S_ISLNK(value_);
    }
};

struct stat_info final {
    uint32_t nlink;
    fspp::mode_t mode;
    fspp::uid_t uid;
    fspp::gid_t gid;
    uint64_t size;
    uint64_t blocks;
    struct timespec atime;
    struct timespec mtime;
    struct timespec ctime;
};

struct statvfs final {
    uint32_t max_filename_length;

    uint32_t blocksize;
    uint64_t num_total_blocks;
    uint64_t num_free_blocks;
    uint64_t num_available_blocks; // free blocks for unprivileged users

    uint64_t num_total_inodes;
    uint64_t num_free_inodes;
    uint64_t num_available_inodes; // free inodes for unprivileged users
};

}

#endif
