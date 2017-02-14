#include <cryfs/cryfs.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

const char *EXISTING_BASEDIR = "/tmp/cryfs-lib-link-test-dir";
const char *NONEXISTENT_BASEDIR = "/nonexistent/basedir";
const char *PASSWORD = "mypassword";
const char *NONEXISTENT_CONFIGFILE = "/nonexistent/configfile";
const uint32_t API_VERSION = 1;

// Call each API function once here and test all linking works fine.

void test_cryfs_load_functions() {
    cryfs_load_context *ctx;
    if (cryfs_success != cryfs_load_init(API_VERSION, &ctx)) {
        printf("Error: cryfs_load_init didn't return success\n");
        exit(EXIT_FAILURE);
    }
    if (cryfs_success == cryfs_load_set_basedir(ctx, NONEXISTENT_BASEDIR, strlen(NONEXISTENT_BASEDIR))) {
        printf("Error: cryfs_load_set_basedir shouldn't have succeeded\n");
        exit(EXIT_FAILURE);
    }
    if (cryfs_success != cryfs_load_set_password(ctx, PASSWORD, strlen(PASSWORD))) {
        printf("Error: cryfs_load_set_password didn't succeed\n");
        exit(EXIT_FAILURE);
    }
    if (cryfs_success == cryfs_load_set_externalconfig(ctx, NONEXISTENT_CONFIGFILE, strlen(NONEXISTENT_CONFIGFILE))) {
        printf("Error: cryfs_load_set_externalconfig shouldn't have succeeded\n");
        exit(EXIT_FAILURE);
    }
    cryfs_mount_handle *handle;
    if (cryfs_success == cryfs_load(ctx, &handle)) {
        printf("Error: cryfs_load shouldn't have succeeded\n");
        exit(EXIT_FAILURE);
    }
    cryfs_load_free(ctx);
}

void test_cryfs_mount_functions() {
    const char rm_command[100] = "\0";
    strcat(rm_command, "rm -rf ");
    strcat(rm_command, EXISTING_BASEDIR);
    system(rm_command);
    const char mk_command[100] = "\0";
    strcat(mk_command, "mkdir ");
    strcat(mk_command, EXISTING_BASEDIR);
    system(mk_command);

    cryfs_load_context *ctx;
    if (cryfs_success != cryfs_load_init(API_VERSION, &ctx)) {
        printf("Error: cryfs_load_init didn't return success\n");
        exit(EXIT_FAILURE);
    }
    if (cryfs_success != cryfs_load_set_basedir(ctx, EXISTING_BASEDIR, strlen(EXISTING_BASEDIR))) {
        printf("Error: cryfs_load_set_basedir should have succeeded\n");
        exit(EXIT_FAILURE);
    }
    if (cryfs_success != cryfs_load_set_password(ctx, PASSWORD, strlen(PASSWORD))) {
        printf("Error: cryfs_load_set_password should have succeeded\n");
        exit(EXIT_FAILURE);
    }
    // TODO Test cryfs_mount_XXX functions
    //cryfs_mount_handle *handle;
    //if (cryfs_success == cryfs_load(ctx, &handle)) {
    //    printf("Error: cryfs_load shouldn't have succeeded\n");
    //    exit(EXIT_FAILURE);
    //}

    // Cleanup
    cryfs_load_free(ctx);
    system(rm_command);
}

int main() {
    test_cryfs_load_functions();
    test_cryfs_mount_functions();

    printf("Success\n");
    exit(EXIT_SUCCESS);
}
