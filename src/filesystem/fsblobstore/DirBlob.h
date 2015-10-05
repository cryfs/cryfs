#pragma once
#ifndef CRYFS_FSBLOBSTORE_DIRBLOB_H_
#define CRYFS_FSBLOBSTORE_DIRBLOB_H_

#include <messmer/blockstore/utils/Key.h>
#include <messmer/cpp-utils/macros.h>
#include <messmer/cpp-utils/data/Data.h>
#include <messmer/fspp/fs_interface/Dir.h>
#include "FsBlob.h"
#include <vector>
#include <mutex>

namespace cryfs {
    namespace fsblobstore {
        class FsBlobStore;

        class DirBlob : public FsBlob {
        public:
            struct Entry {
                Entry(fspp::Dir::EntryType type_, const std::string &name_, const blockstore::Key &key_, mode_t mode_,
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

                fspp::Dir::EntryType type;
                std::string name;
                blockstore::Key key;
                mode_t mode;
                uid_t uid;
                gid_t gid;
            };

            static cpputils::unique_ref<DirBlob> InitializeEmptyDir(cpputils::unique_ref<blobstore::Blob> blob,
                                                                    std::function<off_t (const blockstore::Key&)> getLstatSize);

            DirBlob(cpputils::unique_ref<blobstore::Blob> blob, std::function<off_t (const blockstore::Key&)> getLstatSize);

            virtual ~DirBlob();

            off_t lstat_size() const override;

            void AppendChildrenTo(std::vector<fspp::Dir::Entry> *result) const;

            const Entry &GetChild(const std::string &name) const;

            const Entry &GetChild(const blockstore::Key &key) const;

            void AddChildDir(const std::string &name, const blockstore::Key &blobKey, mode_t mode, uid_t uid,
                             gid_t gid);

            void AddChildFile(const std::string &name, const blockstore::Key &blobKey, mode_t mode, uid_t uid,
                              gid_t gid);

            void AddChildSymlink(const std::string &name, const blockstore::Key &blobKey, uid_t uid, gid_t gid);

            void AddChild(const std::string &name, const blockstore::Key &blobKey, fspp::Dir::EntryType type,
                          mode_t mode,
                          uid_t uid, gid_t gid);

            void RemoveChild(const blockstore::Key &key);

            void flush();

            void statChild(const blockstore::Key &key, struct ::stat *result) const;

            void chmodChild(const blockstore::Key &key, mode_t mode);

            void chownChild(const blockstore::Key &key, uid_t uid, gid_t gid);

            void setLstatSizeGetter(std::function<off_t(const blockstore::Key&)> getLstatSize);

        private:

            const char *readAndAddNextChild(const char *pos, std::vector<Entry> *result) const;

            bool _hasChild(const std::string &name) const;

            void _readEntriesFromBlob();

            void _writeEntriesToBlob();
            static cpputils::Data _writeEntry(const DirBlob::Entry & entry);

            std::vector<DirBlob::Entry>::iterator _findChild(const blockstore::Key &key);

            std::function<off_t (const blockstore::Key&)> _getLstatSize;
            std::vector<Entry> _entries;
            mutable std::mutex _mutex;
            bool _changed;

            DISALLOW_COPY_AND_ASSIGN(DirBlob);
        };

    }
}

#endif
