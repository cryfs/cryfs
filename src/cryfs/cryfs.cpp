#include <cstring>
#include <iostream>
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

cryfs_status cryfs_mount_get_ciphername(cryfs_mount_handle *handle, char *output, size_t max_output_size) {
    // TODO Implement
    static constexpr const char *CIPHERNAME = "aes-256-gcm";
    std::memcpy(output, CIPHERNAME, strlen(CIPHERNAME)+1);
    return cryfs_success;
}

cryfs_status cryfs_mount_set_logfile(cryfs_mount_handle *handle, const char *logfile) {
    //TODO Implement
    return cryfs_success;
}

cryfs_status cryfs_mount_set_unmount_idle(cryfs_mount_handle *handle, uint32_t unmount_idle_sec) {
    //TODO Implement
    return cryfs_success;
}

cryfs_status cryfs_mount(cryfs_mount_handle *handle, const char *mountdir) {
    //TODO Implement
    std::cout << "Heyho, congrats! I'm mounting to " << mountdir << ". Ok, not actually, this is only a dummy..." << std::endl;
    return cryfs_success;
}
