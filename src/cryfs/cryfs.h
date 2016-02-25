#pragma once
#ifndef CRYFS_CRYFS_H
#define CRYFS_CRYFS_H

#include "cryfs_export.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct cryfs_load_context cryfs_load_context;
typedef enum {cryfs_success = 0} cryfs_status;

CRYFS_EXPORT __attribute__((warn_unused_result)) cryfs_status cryfs_load_init(cryfs_load_context **context);
CRYFS_EXPORT __attribute__((warn_unused_result)) const char *cryfs_test(cryfs_load_context *context);
CRYFS_EXPORT void cryfs_load_free(cryfs_load_context *context);

#ifdef __cplusplus
}
#endif

#endif
