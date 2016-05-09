#include <cstring>
#include <iostream>
#include "cryfs.h"
#include "lib/cryfs_load_context.h"

using std::string;

cryfs_status cryfs_load_init(cryfs_load_context **context) {
    *context = new cryfs_load_context;
    return cryfs_success;
}

void cryfs_load_free(cryfs_load_context *context) {
    delete context;
}

cryfs_status cryfs_load_set_basedir(cryfs_load_context *context, const char *basedir, size_t basedir_length) {
    return context->set_basedir(string(basedir, basedir_length));
}

cryfs_status cryfs_load_set_password(cryfs_load_context *context, const char *password, size_t password_length) {
    return context->set_password(string(password, password_length));
}

cryfs_status cryfs_load_set_externalconfig(cryfs_load_context *context, const char *configfile, size_t configfile_length) {
    return context->set_externalconfig(string(configfile, configfile_length));
}

cryfs_status cryfs_load(cryfs_load_context *context, cryfs_mount_handle **handle) {
    return context->load(handle);
}

cryfs_status cryfs_mount_set_run_in_foreground(cryfs_mount_handle *handle, bool run_in_foreground) {
    return handle->set_run_in_foreground(run_in_foreground);
}

cryfs_status cryfs_mount_set_mountdir(cryfs_mount_handle *handle, const char *mountdir, size_t mountdir_length) {
    return handle->set_mountdir(string(mountdir, mountdir_length));
}

cryfs_status cryfs_mount_add_fuse_argument(cryfs_mount_handle *handle, const char *argument, size_t argument_length) {
    return handle->add_fuse_argument(string(argument, argument_length));
}

cryfs_status cryfs_mount_get_ciphername(cryfs_mount_handle *handle, const char **output) {
    *output = handle->get_ciphername();
    return cryfs_success;
}

cryfs_status cryfs_mount_set_logfile(cryfs_mount_handle *handle, const char *logfile, size_t logfile_length) {
    return handle->set_logfile(string(logfile, logfile_length));
}

cryfs_status cryfs_mount_set_unmount_idle(cryfs_mount_handle *handle, uint32_t unmount_idle_sec) {
    return handle->set_unmount_idle(std::chrono::seconds(unmount_idle_sec));
}

cryfs_status cryfs_mount(cryfs_mount_handle *handle) {
    return handle->mount();
}
