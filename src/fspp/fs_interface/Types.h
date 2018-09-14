#pragma once
#ifndef MESSMER_FSPP_FSINTERFACE_TYPES_H_
#define MESSMER_FSPP_FSINTERFACE_TYPES_H_

#include <cstdint>
#include <ctime>

namespace fspp {

struct stat_info final {
    uint32_t nlink;
    uint32_t mode;
    uint32_t uid;
    uint32_t gid;
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
