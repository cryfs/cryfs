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

private:
	static constexpr mode_t S_IFMT_() { return mode_t(0170000); };
	static constexpr mode_t S_IFDIR_() { return mode_t(0040000); };
	static constexpr mode_t S_IFREG_() { return mode_t(0100000); };
	static constexpr mode_t S_IFLNK_() { return mode_t(0120000); };

	static constexpr mode_t S_IRUSR_() { return mode_t(0000400); };
	static constexpr mode_t S_IWUSR_() { return mode_t(0000200); };
	static constexpr mode_t S_IXUSR_() { return mode_t(0000100); };
	static constexpr mode_t S_IRGRP_() { return mode_t(0000040); };
	static constexpr mode_t S_IWGRP_() { return mode_t(0000020); };
	static constexpr mode_t S_IXGRP_() { return mode_t(0000010); };
	static constexpr mode_t S_IROTH_() { return mode_t(0000004); };
	static constexpr mode_t S_IWOTH_() { return mode_t(0000002); };
	static constexpr mode_t S_IXOTH_() { return mode_t(0000001); };

	static constexpr bool S_ISREG_(mode_t mode) {
		return (mode & S_IFMT_()) == S_IFREG_();
	}

	static constexpr bool S_ISDIR_(mode_t mode) {
		return (mode & S_IFMT_()) == S_IFDIR_();
	}

	static constexpr bool S_ISLNK_(mode_t mode) {
		return (mode & S_IFMT_()) == S_IFLNK_();
	}

public:
    constexpr mode_t& addFileFlag() noexcept {
        return *this |= S_IFREG_();
    }

    constexpr mode_t& addDirFlag() noexcept {
        return *this |= S_IFDIR_();
    }

    constexpr mode_t& addSymlinkFlag() noexcept {
        return *this |= S_IFLNK_();
    }

    constexpr mode_t& addUserReadFlag() noexcept {
        return *this |= S_IRUSR_();
    }

    constexpr mode_t& addUserWriteFlag() noexcept {
		return *this |= S_IWUSR_();
    }

    constexpr mode_t& addUserExecFlag() noexcept {
		return *this |= S_IXUSR_();
    }

    constexpr mode_t& addGroupReadFlag() noexcept {
		return *this |= S_IRGRP_();
    }

    constexpr mode_t& addGroupWriteFlag() noexcept {
		return *this |= S_IWGRP_();
    }

    constexpr mode_t& addGroupExecFlag() noexcept {
		return *this |= S_IXGRP_();
    }

    constexpr mode_t& addOtherReadFlag() noexcept {
		return *this |= S_IROTH_();
    }

    constexpr mode_t& addOtherWriteFlag() noexcept {
		return *this |= S_IWOTH_();
    }

    constexpr mode_t& addOtherExecFlag() noexcept {
		return *this |= S_IXOTH_();
    }

    constexpr bool hasFileFlag() const noexcept {
        return S_ISREG_(*this);
    }

    constexpr bool hasDirFlag() const noexcept {
        return S_ISDIR_(*this);
    }

    constexpr bool hasSymlinkFlag() const noexcept {
        return S_ISLNK_(*this);
    }
};

struct openflags_t final : cpputils::value_type::FlagsValueType<openflags_t, int> {
	// TODO Remove default constructor
	constexpr openflags_t() noexcept: FlagsValueType(0) {}

	constexpr explicit openflags_t(int id) noexcept : FlagsValueType(id) {}

	constexpr int value() const noexcept {
		return value_;
	}

	static constexpr openflags_t RDONLY() { return openflags_t(0x0000); };
	static constexpr openflags_t WRONLY() { return openflags_t(0x0001); };
	static constexpr openflags_t RDWR() { return openflags_t(0x0002); };
};

struct num_bytes_t final : cpputils::value_type::QuantityValueType<num_bytes_t, int64_t> {
    // TODO Remove default constructor
    constexpr num_bytes_t() noexcept: QuantityValueType(0) {}

    constexpr explicit num_bytes_t(int64_t id) noexcept: QuantityValueType(id) {}

    constexpr int64_t value() const noexcept {
        return value_;
    }
};

struct stat_info final {
    uint32_t nlink{};
    fspp::mode_t mode;
    fspp::uid_t uid;
    fspp::gid_t gid;
    fspp::num_bytes_t size;
    uint64_t blocks{};
    struct timespec atime{};
    struct timespec mtime{};
    struct timespec ctime{};
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
