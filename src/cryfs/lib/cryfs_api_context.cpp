#include "cryfs_api_context.h"
#include "cryfs_load_context.h"
#include "cryfs_create_context.h"
#include "cryfs_mount_handle.h"

using std::unique_lock;
using std::mutex;
using cpputils::make_unique_ref;

cryfs_api_context::cryfs_api_context()
    : _load_contexts(), _create_contexts() {
}

cryfs_load_context *cryfs_api_context::new_load_context() {
    return _load_contexts.create(this);
}

cryfs_create_context *cryfs_api_context::new_create_context() {
    return _create_contexts.create(this);
}

cryfs_status cryfs_api_context::delete_load_context(cryfs_load_context *context) {
    return _load_contexts.remove(context);
}

cryfs_status cryfs_api_context::delete_create_context(cryfs_create_context *context) {
    return _create_contexts.remove(context);
}
