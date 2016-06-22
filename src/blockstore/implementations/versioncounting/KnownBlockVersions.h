#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_KNOWNBLOCKVERSIONS_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_VERSIONCOUNTING_KNOWNBLOCKVERSIONS_H_

#include <cpp-utils/macros.h>
#include <blockstore/utils/Key.h>
#include <boost/filesystem/path.hpp>
#include <boost/optional.hpp>

namespace blockstore {
    namespace versioncounting {

        class KnownBlockVersions final {
        public:
            KnownBlockVersions(const boost::filesystem::path &stateFilePath);
            KnownBlockVersions(KnownBlockVersions &&rhs);
            ~KnownBlockVersions();

            __attribute__((warn_unused_result))
            bool checkAndUpdateVersion(const Key &key, uint64_t version);

            void updateVersion(const Key &key, uint64_t version);

            uint32_t myClientId() const;

        private:
            std::unordered_map<Key, uint64_t> _knownVersions;
            boost::filesystem::path _stateFilePath;
            uint32_t _myClientId;
            bool _valid;

            static const std::string HEADER;

            void _loadStateFile();
            static void _checkHeader(std::ifstream *file);
            static std::pair<Key, uint64_t> _readEntry(std::ifstream *file);
            static void _checkIsEof(std::ifstream *file);
            void _saveStateFile() const;

            DISALLOW_COPY_AND_ASSIGN(KnownBlockVersions);
        };

    }
}


#endif
