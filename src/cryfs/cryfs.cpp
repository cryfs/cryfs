#include "cryfs.h"

class cryfs_load_context {
public:
    cryfs_load_context(const char *value_): value(value_) {}

    const char *value;
};

cryfs_status cryfs_load_init(cryfs_load_context **context) {
    *context = new cryfs_load_context("Hello Library World!");
    return cryfs_success;
}

void cryfs_load_free(cryfs_load_context *context) {
    delete context;
}

const char *cryfs_test(cryfs_load_context *context) {
    return context->value;
}
