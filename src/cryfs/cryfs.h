#pragma once
#ifndef CRYFS_CRYFS_H
#define CRYFS_CRYFS_H

#include <stddef.h>
#include "cryfs_export.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct cryfs_load_context cryfs_load_context;
typedef struct cryfs_mount_handle cryfs_mount_handle;
typedef enum {
    cryfs_success = 0,
    cryfs_error_BASEDIR_NOT_SET = -1,
    cryfs_error_PASSWORD_NOT_SET = -2
} cryfs_status;

CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_init(cryfs_load_context **context);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_set_basedir(cryfs_load_context *context, const char *basedir);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_set_password(cryfs_load_context *context, const char *password, size_t password_length);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_set_externalconfig(cryfs_load_context *context, const char *configfile);
CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load(cryfs_load_context *context, cryfs_mount_handle **handle);

CRYFS_EXPORT void cryfs_load_free(cryfs_load_context *context);

#ifdef __cplusplus
}
#endif

#endif
