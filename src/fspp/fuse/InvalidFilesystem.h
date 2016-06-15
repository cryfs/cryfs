#pragma once
#ifndef MESSMER_FSPP_FUSE_INVALIDFILESYSTEM_H_
#define MESSMER_FSPP_FUSE_INVALIDFILESYSTEM_H_

#include "Filesystem.h"

namespace fspp {
    namespace fuse {
        class InvalidFilesystem final : public Filesystem {
            int createAndOpenFile(const boost::filesystem::path &, mode_t , uid_t , gid_t ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            int openFile(const boost::filesystem::path &, int ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void flush(int ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void closeFile(int ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void lstat(const boost::filesystem::path &, struct ::stat *) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void fstat(int , struct ::stat *) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void chmod(const boost::filesystem::path &, mode_t ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void chown(const boost::filesystem::path &, uid_t , gid_t ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void truncate(const boost::filesystem::path &, off_t ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void ftruncate(int , off_t ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            size_t read(int , void *, size_t , off_t ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void write(int , const void *, size_t , off_t ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void fsync(int ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void fdatasync(int ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void access(const boost::filesystem::path &, int ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void mkdir(const boost::filesystem::path &, mode_t , uid_t , gid_t ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void rmdir(const boost::filesystem::path &) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void unlink(const boost::filesystem::path &) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void rename(const boost::filesystem::path &, const boost::filesystem::path &) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void utimens(const boost::filesystem::path &, timespec , timespec ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void statfs(const boost::filesystem::path &, struct statvfs *) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            cpputils::unique_ref<std::vector<Dir::Entry>> readDir(const boost::filesystem::path &) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void createSymlink(const boost::filesystem::path &, const boost::filesystem::path &, uid_t , gid_t ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

            void readSymlink(const boost::filesystem::path &, char *, size_t ) override {
                throw std::logic_error("Filesystem not initialized yet");
            }

        };
    }
}

#endif
