#include "testutils/C_Library_Test.h"
#include <gitversion/gitversion.h>
#include "testutils/FilesystemHelper.h"

using cryfs::CryConfig;
using cryfs::CryConfigFile;
using cryfs::CryDevice;
using cryfs::CryCiphers;
using cpputils::TempDir;
using cpputils::TempFile;
using cpputils::Random;
using cpputils::AES256_GCM;
using cpputils::SCrypt;
using cpputils::unique_ref;
using cpputils::make_unique_ref;
using std::shared_ptr;
using blockstore::ondisk::OnDiskBlockStore2;
namespace bf = boost::filesystem;

class Unmount_Test : public C_Library_Test {
public:
    TempDir basedir;
    TempDir mountdir;

    void create_and_mount_filesystem() {
        create_filesystem(basedir.path());

        cryfs_load_context *context = nullptr;
        cryfs_mount_handle *handle = nullptr;
        EXPECT_SUCCESS(cryfs_load_init(api, &context));
        EXPECT_SUCCESS(cryfs_load_set_basedir(context, basedir.path().native().c_str(), basedir.path().native().size()));
        EXPECT_SUCCESS(cryfs_load_set_password(context, PASSWORD.c_str(), PASSWORD.size()));
        EXPECT_SUCCESS(cryfs_load(context, &handle));
        EXPECT_SUCCESS(cryfs_mount_set_mountdir(handle, mountdir.path().native().c_str(), mountdir.path().native().size()));
        EXPECT_SUCCESS(cryfs_mount(handle));
        EXPECT_SUCCESS(cryfs_load_free(&context));
    }

    cryfs_status unmount() {
        return cryfs_unmount(api, mountdir.path().native().c_str(), mountdir.path().native().size());
    }
};

TEST_F(Unmount_Test, mount_and_unmount) {
  create_and_mount_filesystem();
  EXPECT_SUCCESS(unmount());
}

TEST_F(Unmount_Test, unmount_when_not_mounted) {
  EXPECT_EQ(cryfs_error_UNMOUNT_FAILED, unmount());
}
