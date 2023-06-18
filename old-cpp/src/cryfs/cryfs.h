#pragma once
#ifndef MESSMER_CRYFS_CRYFS_H
#define MESSMER_CRYFS_CRYFS_H

#ifdef __cplusplus
extern "C" {
#endif

struct cryfs_load_handle;

cryfs_load_handle *cryfs_load_init();
void cryfs_load_free(cryfs_load_handle *handle);
const char *cryfs_test(cryfs_load_handle *handle);

#ifdef __cplusplus
}
#endif

#endif
