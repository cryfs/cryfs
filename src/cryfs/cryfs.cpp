#include "cryfs.h"
#include "lib/cryfs_load_context.h"

cryfs_status cryfs_load_init(cryfs_load_context **context) {
    *context = new cryfs_load_context;
    return cryfs_success;
}

void cryfs_load_free(cryfs_load_context *context) {
    delete context;
}

cryfs_status cryfs_load_set_basedir(cryfs_load_context *context, const char *basedir) {
    return context->set_basedir(basedir);
}

cryfs_status cryfs_load_set_password(cryfs_load_context *context, const char *password, size_t password_length) {
    return context->set_password(password, password_length);
}

cryfs_status cryfs_load_set_externalconfig(cryfs_load_context *context, const char *configfile) {
    return context->set_externalconfig(configfile);
}

cryfs_status cryfs_load(cryfs_load_context *context, cryfs_mount_handle **handle) {
    return context->load(handle);
}
