#pragma once
#ifndef MESSMER_CRYFS_CRYFS_H
#define MESSMER_CRYFS_CRYFS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct cryfs_load_context cryfs_load_context;
typedef enum {cryfs_success = 0} cryfs_status;

cryfs_status cryfs_load_init(cryfs_load_context **context) __attribute__((warn_unused_result));
void cryfs_load_free(cryfs_load_context *context);
const char *cryfs_test(cryfs_load_context *context) __attribute__((warn_unused_result));

#ifdef __cplusplus
}
#endif

#endif
