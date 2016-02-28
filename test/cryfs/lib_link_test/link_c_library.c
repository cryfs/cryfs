#include <cryfs/cryfs.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

const char *NONEXISTENT_BASEDIR = "/nonexistent/basedir";
const char *PASSWORD = "mypassword";
const char *NONEXISTENT_CONFIGFILE = "/nonexistent/configfile";

int main() {
    // Call each API function once here and test all linking works fine.

    cryfs_load_context *ctx;
    if (cryfs_success != cryfs_load_init(&ctx)) {
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

    printf("Success\n");
    exit(EXIT_SUCCESS);
}
