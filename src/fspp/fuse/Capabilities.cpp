#include "Capabilities.h"
#include "params.h"

namespace fspp {
namespace fuse {

uint32_t unsupportedCapabilities() {
  return
      // CryFile::open() ignores the open flags, so it doesn't truncate a file that was opened with
      // O_TRUNC. With this capability the kernel would stop sending the separate truncate request
      // and expect open() to have done it.
      FUSE_CAP_ATOMIC_O_TRUNC

      // We don't implement locking. libfuse already clears this itself when a file system has no
      // lock handler (fuse_fs_init() in libfuse's lib/fuse.c), but saying so explicitly keeps the
      // intent in one place and keeps it off if libfuse ever changes that default.
      | FUSE_CAP_POSIX_LOCKS

      // libfuse 2 had no such capability, libfuse 3 wants it by default. It makes the kernel
      // refresh attributes on every read and drop a file's page cache whenever the reported mtime
      // differs from the cached one, and CryFS stamps mtime from its own clock, so the two rarely
      // match. Measured on a mount, the cost of leaving it on is an extra getattr per file per
      // attribute timeout on read-heavy workloads, and each getattr is a full path traversal in
      // CryFS. It buys us nothing in return: CryFS does not support concurrent access from several
      // clients, so the data the kernel has cached cannot change behind its back.
      | FUSE_CAP_AUTO_INVAL_DATA

      // Our readdir() only fills in st_mode and never passes FUSE_FILL_DIR_PLUS, so answering
      // readdirplus requests would only make the replies bigger without saving the kernel any
      // lookups. On libfuse 3.0 to 3.10.2 it is worse than that: the non-plus branch there doesn't
      // copy st_mode into the entry, so every entry comes back as DT_UNKNOWN and tools like
      // 'find -type' or 'ls --color' fall back to one lstat per entry - again a full path traversal
      // each. If readdir ever fills in complete stat data, this is the capability to re-enable.
      | FUSE_CAP_READDIRPLUS | FUSE_CAP_READDIRPLUS_AUTO

      // A file system that accepts this promises to clear S_ISUID/S_ISGID itself on write, truncate
      // and chown. CryFS has no such code, so we must not claim it. What has been saving us is a
      // libfuse quirk: 3.0 to 3.16 never copy the bit into the INIT reply, and 3.17 removed it from
      // the defaults. Clearing it makes that independent of the libfuse version: the kernel keeps
      // stripping those bits for us, which is what CryFS relied on under libfuse 2 as well.
      | FUSE_CAP_HANDLE_KILLPRIV;

  // Note that FUSE_CAP_EXPORT_SUPPORT is deliberately *not* in this list. It is implemented by
  // libfuse's own high level library, which enables it by default in both libfuse 2 (fuse_lib_init()
  // in lib/fuse.c) and libfuse 3, so clearing it would newly turn off NFS export for mounts that had
  // it before.
}

uint32_t removeUnsupportedCapabilities(uint32_t want) {
  return want & ~unsupportedCapabilities();
}

}
}
