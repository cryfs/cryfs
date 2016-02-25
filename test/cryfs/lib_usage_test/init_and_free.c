#include <cryfs/cryfs.h>
#include <stdio.h>
#include <stdlib.h>

int main() {
    cryfs_load_context *ctx;
    if (cryfs_success != cryfs_load_init(&ctx)) {
        printf("Error calling cryfs_load_init");
        exit(1);
    }
    cryfs_load_free(ctx);
    printf("Success\n");
    exit(0);
}
