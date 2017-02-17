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
    cryfs_error_UNKNOWN_ERROR = -1,
    cryfs_error_INVALID_CONTEXT = -2,
    cryfs_error_UNSUPPORTED_API_VERSION = -3,
    cryfs_error_BASEDIR_NOT_SET = -4,
    cryfs_error_PASSWORD_NOT_SET = -5,
    cryfs_error_CONFIGFILE_DOESNT_EXIST = -6,
    cryfs_error_CONFIGFILE_NOT_READABLE = -7,
    cryfs_error_BASEDIR_DOESNT_EXIST = -8,
    cryfs_error_BASEDIR_INACCESSIBLE = -9,
    cryfs_error_FILESYSTEM_INCOMPATIBLE_VERSION = -10,
    cryfs_error_FILESYSTEM_INVALID = -11,
    cryfs_error_DECRYPTION_FAILED = -12,
    cryfs_error_MOUNTDIR_DOESNT_EXIST = -13,
    cryfs_error_MOUNTDIR_NOT_SET = -14,
    cryfs_error_MOUNTDIR_INACCESSIBLE = -15,
    cryfs_error_INVALID_LOGFILE = -16,
    cryfs_error_LOGFILE_NOT_WRITABLE = -17,
    cryfs_error_UNMOUNT_FAILED = -18
} cryfs_status;

typedef struct cryfs_api_context cryfs_api_context;
typedef struct cryfs_load_context cryfs_load_context;
typedef struct cryfs_create_context cryfs_create_context;
typedef struct cryfs_mount_handle cryfs_mount_handle;

// Initialize and free the API
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_init(uint32_t api_version, cryfs_api_context **result);
CRYFS_EXPORT void cryfs_free(cryfs_api_context **api_context);

// Loading a file system
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_init(cryfs_api_context *api_context, cryfs_load_context **result);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_set_basedir(cryfs_load_context *context, const char *basedir, size_t basedir_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_set_password(cryfs_load_context *context, const char *password, size_t password_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_set_externalconfig(cryfs_load_context *context, const char *configfile, size_t configfile_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load(cryfs_load_context *context, cryfs_mount_handle **result); // result can be nullptr if you don't want a mount handle
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_free(cryfs_load_context **context);

// Creating a file system
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_create_init(cryfs_api_context *api_context, cryfs_create_context **result);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_create_set_basedir(cryfs_create_context *context, const char *basedir, size_t basedir_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_create_set_cipher(cryfs_create_context *context, const char *cipher, size_t cipher_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_create_set_password(cryfs_create_context *context, const char *password, size_t password_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_create_set_externalconfig(cryfs_create_context *context, const char *configfile, size_t configfile_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_create(cryfs_create_context *context, cryfs_mount_handle **result); // result can be nullptr if you don't want a mount handle
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_create_free(cryfs_create_context **context);

// Mounting a file system
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_get_ciphername(cryfs_mount_handle *handle, const char **result);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_set_run_in_foreground(cryfs_mount_handle *handle, bool run_in_foreground);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_set_mountdir(cryfs_mount_handle *handle, const char *mountdir, size_t mountdir_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_set_logfile(cryfs_mount_handle *handle, const char *logfile, size_t logfile_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_set_unmount_idle_milliseconds(cryfs_mount_handle *handle, uint32_t unmount_idle_milliseconds);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount_add_fuse_argument(cryfs_mount_handle *handle, const char *argument, size_t argument_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_mount(cryfs_mount_handle *handle);

// Unmounting a file system
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_unmount(cryfs_api_context *api_context, const char *mountdir, size_t mountdir_length);

#ifdef __cplusplus
}
#endif

// Remove all potential preprocessor defines so they don't clutter client applications
#undef CRYFS_EXPORT
#undef CRYFS_NO_EXPORT
#undef CRYFS_DEPRECATED
#undef CRYFS_DEPRECATED_EXPORT
#undef CRYFS_DEPRECATED_NO_EXPORT

#endif
