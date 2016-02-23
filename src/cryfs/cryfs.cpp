#include "cryfs.h"

class cryfs_load_handle {
public:
    cryfs_load_handle(const char *value_): value(value_) {}

    const char *value;
};

cryfs_load_handle *cryfs_load_init() {
    return new cryfs_load_handle("Hello Library World!");
}

void cryfs_load_free(cryfs_load_handle *handle) {
    delete handle;
}

const char *cryfs_test(cryfs_load_handle *handle) {
    return handle->value;
}
