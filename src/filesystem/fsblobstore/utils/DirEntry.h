#pragma once
#ifndef CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRY_H
#define CRYFS_FILESYSTEM_FSBLOBSTORE_UTILS_DIRENTRY_H

#include <messmer/blockstore/utils/Key.h>
#include <messmer/fspp/fs_interface/Dir.h>

namespace cryfs {
    namespace fsblobstore {

        struct DirEntry {
            DirEntry(fspp::Dir::EntryType type_, const std::string &name_, const blockstore::Key &key_, mode_t mode_,
                  uid_t uid_, gid_t gid_) : type(type_), name(name_), key(key_), mode(mode_), uid(uid_), gid(gid_) {
                switch (type) {
                    case fspp::Dir::EntryType::FILE:
                        mode |= S_IFREG;
                        break;
                    case fspp::Dir::EntryType::DIR:
                        mode |= S_IFDIR;
                        break;
                    case fspp::Dir::EntryType::SYMLINK:
                        mode |= S_IFLNK;
                        break;
                }
                ASSERT((S_ISREG(mode) && type == fspp::Dir::EntryType::FILE) ||
                       (S_ISDIR(mode) && type == fspp::Dir::EntryType::DIR) ||
                       (S_ISLNK(mode) && type == fspp::Dir::EntryType::SYMLINK), "Unknown mode in entry");
            }

            void serialize(uint8_t* dest) const;
            size_t serializedSize() const;
            static const char *deserializeAndAddToVector(const char *pos, std::vector<DirEntry> *result);

            fspp::Dir::EntryType type;
            std::string name;
            blockstore::Key key;
            mode_t mode;
            uid_t uid;
            gid_t gid;
        };

    }
}

#endif
