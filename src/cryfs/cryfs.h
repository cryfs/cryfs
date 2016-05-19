#pragma once
#ifndef CRYFS_CRYFS_H
#define CRYFS_CRYFS_H

#include <stddef.h>
#include <stdint.h>
#include "cryfs_export.h"
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * WARNING!
 * This API isn't final yet. Expect it to be modified (and to be incompatible) in future versions.
 */

typedef enum {
    cryfs_success = 0,
    cryfs_error_BASEDIR_NOT_SET = -1,
    cryfs_error_PASSWORD_NOT_SET = -2,
    cryfs_error_CONFIGFILE_DOESNT_EXIST = -3,
    cryfs_error_BASEDIR_DOESNT_EXIST = -4,
    cryfs_error_FILESYSTEM_INCOMPATIBLE_VERSION = -5,
    cryfs_error_FILESYSTEM_INVALID = -6,
    cryfs_error_DECRYPTION_FAILED = -7,
    cryfs_error_MOUNTDIR_DOESNT_EXIST = -8,
    cryfs_error_MOUNTDIR_NOT_SET = -9,
    cryfs_error_MOUNTHANDLE_ALREADY_USED = -10,
    cryfs_error_INVALID_LOGFILE = -11,
    cryfs_error_UNMOUNT_FAILED = -12
} cryfs_status;

typedef struct cryfs_load_context cryfs_load_context;
typedef struct cryfs_mount_handle cryfs_mount_handle;

// Loading a file system
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_init(cryfs_load_context **context);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_set_basedir(cryfs_load_context *context, const char *basedir, size_t basedir_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_set_password(cryfs_load_context *context, const char *password, size_t password_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_set_externalconfig(cryfs_load_context *context, const char *configfile, size_t configfile_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load(cryfs_load_context *context, cryfs_mount_handle **handle);
CRYFS_EXPORT void cryfs_load_free(cryfs_load_context *context);

// Mounting a file system
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_get_ciphername(cryfs_mount_handle *handle, const char **output);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_set_run_in_foreground(cryfs_mount_handle *handle, bool run_in_foreground);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_set_mountdir(cryfs_mount_handle *handle, const char *mountdir, size_t mountdir_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_set_logfile(cryfs_mount_handle *handle, const char *logfile, size_t logfile_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_set_unmount_idle(cryfs_mount_handle *handle, uint32_t unmount_idle_sec);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_add_fuse_argument(cryfs_mount_handle *handle, const char *argument, size_t argument_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount(cryfs_mount_handle *handle);

// Unmounting a file system
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_unmount(const char *mountdir, size_t mountdir_length);

#ifdef __cplusplus
}
#endif

#endif
