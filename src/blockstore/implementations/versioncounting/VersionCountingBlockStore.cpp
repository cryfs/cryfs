#include "VersionCountingBlockStore.h"

using cpputils::unique_ref;
using cpputils::make_unique_ref;
using cpputils::Data;
using boost::none;
namespace bf = boost::filesystem;

namespace blockstore {
    namespace versioncounting {

#ifndef CRYFS_NO_COMPATIBILITY
        void VersionCountingBlockStore::migrateFromBlockstoreWithoutVersionNumbers(BlockStore *baseBlockStore, const bf::path &integrityFilePath) {
            std::cout << "Migrating file system for integrity features..." << std::flush;
            KnownBlockVersions knownBlockVersions(integrityFilePath);
            baseBlockStore->forEachBlock([&baseBlockStore, &knownBlockVersions] (const Key &key) {
                auto block =  baseBlockStore->load(key);
                ASSERT(block != none, "Couldn't load block for migration");
                VersionCountingBlock::migrateFromBlockstoreWithoutVersionNumbers(std::move(*block), &knownBlockVersions);
            });
            std::cout << "done" << std::endl;
        }
#endif

    }
}
