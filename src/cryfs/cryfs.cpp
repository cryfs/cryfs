#include <cstring>
#include <iostream>
#include "cryfs.h"
#include "lib/cryfs_api_context.h"
#include "lib/cryfs_load_context.h"
#include "lib/cryfs_create_context.h"
#include "lib/cryfs_mount_handle.h"
#include "lib/cryfs_unmounter.h"
#include <cpp-utils/logging/logging.h>

using namespace cpputils::logging;

using std::string;
using std::function;

namespace {
__attribute__((warn_unused_result)) cryfs_status catchAllExceptions(function<cryfs_status()> func) {
    try {
        return func();
    } catch (const std::exception &e) {
        LOG(ERR, "Unknown exception: {}", e.what());
        return cryfs_error_UNKNOWN_ERROR;
    } catch (...) {
        LOG(ERR, "Unknown error");
        return cryfs_error_UNKNOWN_ERROR;
    }
};

void catchAllExceptionsNoReturn(function<void()> func) {
    try {
        func();
    } catch (const std::exception &e) {
        LOG(ERR, "Unknown exception: {}", e.what());
    } catch (...) {
        LOG(ERR, "Unknown error");
    }
};
}

cryfs_status cryfs_init(uint32_t api_version, cryfs_api_context **result) {
    return catchAllExceptions([&] {
        if (1 != api_version) {
          *result = nullptr;
          return cryfs_error_UNSUPPORTED_API_VERSION;
        }
        *result = new cryfs_api_context;
        return cryfs_success;
    });
}

void cryfs_free(cryfs_api_context **api_context) {
    return catchAllExceptionsNoReturn([&] {
        if (nullptr != api_context && nullptr != *api_context) {
            delete *api_context;
            *api_context = nullptr;
        }
    });
}

cryfs_status cryfs_load_init(cryfs_api_context *api_context, cryfs_load_context **result) {
    return catchAllExceptions([&] {
        if (nullptr == api_context) {
            return cryfs_error_INVALID_CONTEXT;
        }
        *result = api_context->new_load_context();
        return cryfs_success;
    });
}

cryfs_status cryfs_load_free(cryfs_load_context **context) {
    return catchAllExceptions([&] {
        if (nullptr == context || nullptr == *context) {
            return cryfs_error_INVALID_CONTEXT;
        }
        auto result = (*context)->free();
        if (cryfs_success == result) {
          *context = nullptr;
        }
        return result;
    });
}

cryfs_status cryfs_load_set_basedir(cryfs_load_context *context, const char *basedir, size_t basedir_length) {
    return catchAllExceptions([&] {
        if (nullptr == context) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return context->set_basedir(string(basedir, basedir_length));
    });
}

cryfs_status cryfs_load_set_password(cryfs_load_context *context, const char *password, size_t password_length) {
    return catchAllExceptions([&] {
        if (nullptr == context) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return context->set_password(string(password, password_length));
    });
}

cryfs_status cryfs_load_set_externalconfig(cryfs_load_context *context, const char *configfile, size_t configfile_length) {
    return catchAllExceptions([&] {
        if (nullptr == context) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return context->set_externalconfig(string(configfile, configfile_length));
    });
}

cryfs_status cryfs_load_set_localstatedir(cryfs_load_context *context, const char *localstatedir, size_t localstatedir_length) {
    return catchAllExceptions([&] {
        if (nullptr == context) {
            return cryfs_error_INVALID_CONTEXT;
        }
        return context->set_localstatedir(string(localstatedir, localstatedir_length));
    });
}

cryfs_status cryfs_load(cryfs_load_context *context, cryfs_mount_handle **handle) {
    return catchAllExceptions([&] {
        if (nullptr == context) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return context->load(handle);
    });
}

cryfs_status cryfs_create_init(cryfs_api_context *api_context, cryfs_create_context **context) {
    return catchAllExceptions([&] {
        if (nullptr == api_context) {
          return cryfs_error_INVALID_CONTEXT;
        }
        *context = api_context->new_create_context();
        return cryfs_success;
    });
}

cryfs_status cryfs_create_set_basedir(cryfs_create_context *context, const char *basedir, size_t basedir_length) {
    return catchAllExceptions([&] {
        if (nullptr == context) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return context->set_basedir(string(basedir, basedir_length));
    });
}

cryfs_status cryfs_create_set_cipher(cryfs_create_context *context, const char *cipher, size_t cipher_length) {
    return catchAllExceptions([&] {
        if (nullptr == context) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return context->set_cipher(string(cipher, cipher_length));
    });
}

cryfs_status cryfs_create_set_password(cryfs_create_context *context, const char *password, size_t password_length) {
    return catchAllExceptions([&] {
        if (nullptr == context) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return context->set_password(string(password, password_length));
    });
}

cryfs_status cryfs_create_set_externalconfig(cryfs_create_context *context, const char *configfile, size_t configfile_length) {
    return catchAllExceptions([&] {
        if (nullptr == context) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return context->set_externalconfig(string(configfile, configfile_length));
    });
}

cryfs_status cryfs_create(cryfs_create_context *context, cryfs_mount_handle **handle) {
    return catchAllExceptions([&] {
        if (nullptr == context) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return context->create(handle);
    });
}

cryfs_status cryfs_create_free(cryfs_create_context **context) {
    return catchAllExceptions([&] {
        if (nullptr == context || nullptr == *context) {
            return cryfs_error_INVALID_CONTEXT;
        }
        auto result = (*context)->free();
        if (cryfs_success == result) {
          *context = nullptr;
        }
        return result;
    });
}

cryfs_status cryfs_mount_set_run_in_foreground(cryfs_mount_handle *handle, bool run_in_foreground) {
    return catchAllExceptions([&] {
        if (nullptr == handle) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return handle->set_run_in_foreground(run_in_foreground);
    });
}

cryfs_status cryfs_mount_set_mountdir(cryfs_mount_handle *handle, const char *mountdir, size_t mountdir_length) {
    return catchAllExceptions([&] {
        if (nullptr == handle) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return handle->set_mountdir(string(mountdir, mountdir_length));
    });
}

cryfs_status cryfs_mount_add_fuse_argument(cryfs_mount_handle *handle, const char *argument, size_t argument_length) {
    return catchAllExceptions([&] {
        if (nullptr == handle) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return handle->add_fuse_argument(string(argument, argument_length));
    });
}

cryfs_status cryfs_mount_get_ciphername(cryfs_mount_handle *handle, const char **output) {
    return catchAllExceptions([&] {
        if (nullptr == handle) {
          return cryfs_error_INVALID_CONTEXT;
        }
        *output = handle->get_ciphername();
        return cryfs_success;
    });
}

cryfs_status cryfs_mount_set_logfile(cryfs_mount_handle *handle, const char *logfile, size_t logfile_length) {
    return catchAllExceptions([&] {
        if (nullptr == handle) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return handle->set_logfile(string(logfile, logfile_length));
    });
}

cryfs_status cryfs_mount_set_unmount_idle_milliseconds(cryfs_mount_handle *handle, uint32_t unmount_idle_milliseconds) {
    return catchAllExceptions([&] {
        if (nullptr == handle) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return handle->set_unmount_idle(boost::chrono::milliseconds(unmount_idle_milliseconds));
    });
}

cryfs_status cryfs_mount(cryfs_mount_handle *handle) {
    return catchAllExceptions([&] {
        if (nullptr == handle) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return handle->mount();
    });
}

cryfs_status cryfs_unmount(cryfs_api_context *api_context, const char *mountdir, size_t mountdir_length) {
    return catchAllExceptions([&] {
        if (nullptr == api_context) {
          return cryfs_error_INVALID_CONTEXT;
        }
        return cryfs::cryfs_unmounter::unmount(string(mountdir, mountdir_length));
    });
}
