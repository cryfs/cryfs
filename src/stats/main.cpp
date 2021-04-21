#include <iostream>
#include <boost/filesystem.hpp>
#include <cryfs/impl/config/CryConfigLoader.h>
#include <cryfs/impl/config/CryPasswordBasedKeyProvider.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore2.h>
#include <blockstore/implementations/readonly/ReadOnlyBlockStore2.h>
#include <blockstore/implementations/integrity/IntegrityBlockStore2.h>
#include <blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNodeStore.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataInnerNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataLeafNode.h>
#include <blobstore/implementations/onblocks/BlobStoreOnBlocks.h>
#include <cryfs/impl/filesystem/fsblobstore/FsBlobStore.h>
#include <cryfs/impl/filesystem/fsblobstore/DirBlob.h>
#include <cryfs/impl/filesystem/CryDevice.h>
#include <cpp-utils/io/IOStreamConsole.h>
#include <cpp-utils/system/homedir.h>
#include "traversal.h"

#include <set>

using std::endl;
using std::cout;
using std::set;
using std::flush;
using std::vector;
using boost::none;
using boost::filesystem::path;

using namespace cryfs;
using namespace cpputils;
using namespace blockstore;
using namespace blockstore::ondisk;
using namespace blockstore::readonly;
using namespace blockstore::integrity;
using namespace blockstore::lowtohighlevel;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;
using namespace cryfs::fsblobstore;

using namespace cryfs_stats;

void printNode(unique_ref<DataNode> node) {
    std::cout << "BlockId: " << node->blockId().ToString() << ", Depth: " << static_cast<int>(node->depth()) << " ";
    auto innerNode = dynamic_pointer_move<DataInnerNode>(node);
    if (innerNode != none) {
        std::cout << "Type: inner\n";
        return;
    }
    auto leafNode = dynamic_pointer_move<DataLeafNode>(node);
    if (leafNode != none) {
        std::cout << "Type: leaf\n";
        return;
    }
}

unique_ref<BlockStore> makeBlockStore(const path& basedir, const CryConfigLoader::ConfigLoadResult& config, LocalStateDir& localStateDir) {
    auto onDiskBlockStore = make_unique_ref<OnDiskBlockStore2>(basedir);
    auto readOnlyBlockStore = make_unique_ref<ReadOnlyBlockStore2>(std::move(onDiskBlockStore));
    auto encryptedBlockStore = CryCiphers::find(config.configFile->config()->Cipher()).createEncryptedBlockstore(std::move(readOnlyBlockStore), config.configFile->config()->EncryptionKey());
    auto statePath = localStateDir.forFilesystemId(config.configFile->config()->FilesystemId());
    auto integrityFilePath = statePath / "integritydata";
    auto onIntegrityViolation = [] () {
        std::cerr << "Warning: Integrity violation encountered" << std::endl;
    };
    auto integrityBlockStore = make_unique_ref<IntegrityBlockStore2>(std::move(encryptedBlockStore), integrityFilePath, config.myClientId, false, true, onIntegrityViolation);
    return make_unique_ref<LowToHighLevelBlockStore>(std::move(integrityBlockStore));
}

struct AccumulateBlockIds final {
public:
    auto callback() {
        return [this] (const BlockId& id) {
            _blockIds.push_back(id);
        };
    }

    const std::vector<BlockId>& blockIds() const {
        return _blockIds;
    }

    void reserve(size_t size) {
        _blockIds.reserve(size);
    }

private:
    std::vector<BlockId> _blockIds;
};

class ProgressBar final {
public:
    ProgressBar(size_t numBlocks): _currentBlock(0), _numBlocks(numBlocks) {}

    auto callback() {
        return [this] (const BlockId&) {
            cout << "\r" << (++_currentBlock) << "/" << _numBlocks << flush;
        };
    }
private:
    size_t _currentBlock;
    size_t _numBlocks;
};

std::vector<BlockId> getKnownBlobIds(const path& basedir, const CryConfigLoader::ConfigLoadResult& config, LocalStateDir& localStateDir) {
    auto blockStore = makeBlockStore(basedir, config, localStateDir);
    auto fsBlobStore = make_unique_ref<FsBlobStore>(make_unique_ref<BlobStoreOnBlocks>(std::move(blockStore), config.configFile->config()->BlocksizeBytes()));

    std::vector<BlockId> result;
    AccumulateBlockIds knownBlobIds;
    cout << "Listing all file system entities (i.e. blobs)..." << flush;
    auto rootId = BlockId::FromString(config.configFile->config()->RootBlob());
    forEachReachableBlob(fsBlobStore.get(), rootId, {knownBlobIds.callback()});
    cout << "done" << endl;

    return knownBlobIds.blockIds();
}

std::vector<BlockId> getKnownBlockIds(const path& basedir, const CryConfigLoader::ConfigLoadResult& config, LocalStateDir& localStateDir) {
    auto knownBlobIds = getKnownBlobIds(basedir, config, localStateDir);

    auto blockStore = makeBlockStore(basedir, config, localStateDir);
    auto nodeStore = make_unique_ref<DataNodeStore>(std::move(blockStore), config.configFile->config()->BlocksizeBytes());
    AccumulateBlockIds knownBlockIds;
    const uint32_t numNodes = nodeStore->numNodes();
    knownBlockIds.reserve(numNodes);
    cout << "Listing all blocks used by these file system entities..." << endl;
    for (const auto& blobId : knownBlobIds) {
        forEachReachableBlockInBlob(nodeStore.get(), blobId, {
            ProgressBar(numNodes).callback(),
            knownBlockIds.callback()
        });
    }
    std::cout << "...done" << endl;
    return knownBlockIds.blockIds();
}

set<BlockId> getAllBlockIds(const path& basedir, const CryConfigLoader::ConfigLoadResult& config, LocalStateDir& localStateDir) {
    auto blockStore = makeBlockStore(basedir, config, localStateDir);
    AccumulateBlockIds allBlockIds;
    allBlockIds.reserve(blockStore->numBlocks());
    forEachBlock(blockStore.get(), {allBlockIds.callback()});
    return set<BlockId>(allBlockIds.blockIds().begin(), allBlockIds.blockIds().end());
}

void printConfig(const CryConfig& config) {
    std::cout
        << "----------------------------------------------------"
        << "\nFilesystem configuration:"
        << "\n----------------------------------------------------"
        << "\n- Filesystem format version: " << config.Version()
        << "\n- Created with: CryFS " << config.CreatedWithVersion()
        << "\n- Last opened with: CryFS " << config.LastOpenedWithVersion()
        << "\n- Cipher: " << config.Cipher()
        << "\n- Blocksize: " << config.BlocksizeBytes() << " bytes"
        << "\n- Filesystem Id: " << config.FilesystemId().ToString()
        << "\n- Root Blob Id: " << config.RootBlob();
    if (config.missingBlockIsIntegrityViolation()) {
        ASSERT(config.ExclusiveClientId() != boost::none, "ExclusiveClientId must be set if missingBlockIsIntegrityViolation");
        std::cout << "\n- Extended integrity measures: enabled."
               "\n  - Exclusive client id: " << *config.ExclusiveClientId();
    } else {
        ASSERT(config.ExclusiveClientId() == boost::none, "ExclusiveClientId must be unset if !missingBlockIsIntegrityViolation");
        std::cout << "\n- Extended integrity measures: disabled.";
    }
#ifndef CRYFS_NO_COMPATIBILITY
    std::cout << "\n- Has parent pointers: " << (config.HasParentPointers() ? "yes" : "no");
    std::cout << "\n- Has version numbers: " << (config.HasVersionNumbers() ? "yes" : "no");
#endif
    std::cout << "\n----------------------------------------------------\n";
}

int main(int argc, char* argv[]) {
    if (argc != 2) {
        std::cerr << "Usage: cryfs-stats [basedir]" << std::endl;
        exit(1);
    }
    path basedir = argv[1];
    std::cout << "Calculating stats for filesystem at " << basedir << std::endl;

    auto console = std::make_shared<cpputils::IOStreamConsole>();

    console->print("Loading config\n");
    auto askPassword = [console] () {
        return console->askPassword("Password: ");
    };
    unique_ref<CryKeyProvider> keyProvider = make_unique_ref<CryPasswordBasedKeyProvider>(
        console,
        askPassword,
        askPassword,
        make_unique_ref<SCrypt>(SCrypt::DefaultSettings)
    );

    auto config_path = basedir / "cryfs.config";
    LocalStateDir localStateDir(cpputils::system::HomeDirectory::getXDGDataDir() / "cryfs");
    CryConfigLoader config_loader(console, Random::OSRandom(), std::move(keyProvider), localStateDir, boost::none, boost::none, boost::none);

    auto config = config_loader.load(config_path, false, true, CryConfigFile::Access::ReadOnly);
    if (config.is_left()) {
        switch (config.left()) {
            case CryConfigFile::LoadError::ConfigFileNotFound:
                throw std::runtime_error("Error loading config file: Config file not found. Are you sure this is a valid CryFS file system?");
            case CryConfigFile::LoadError::DecryptionFailed:
                throw std::runtime_error("Error loading config file: Decryption failed. Did you maybe enter a wrong password?");
        }
    }
    const auto& config_ = config.right().configFile->config();
    std::cout << "Loading filesystem" << std::endl;
    printConfig(*config_);
#ifndef CRYFS_NO_COMPATIBILITY
    const bool is_correct_format = config_->Version() == CryConfig::FilesystemFormatVersion && config_->HasParentPointers() && config_->HasVersionNumbers();
#else
    const bool is_correct_format = config_->Version() == CryConfig::FilesystemFormatVersion;
#endif
    if (!is_correct_format) {
        std::cerr << "The filesystem is not in the 0.10 format. It needs to be migrated. The cryfs-stats tool unfortunately can't handle this, please mount and unmount the filesystem once." << std::endl;
        exit(1);
    }

    cout << "Listing all blocks..." << flush;
    set<BlockId> unaccountedBlocks = getAllBlockIds(basedir, config.right(), localStateDir);
    cout << "done" << endl;

    vector<BlockId> accountedBlocks = getKnownBlockIds(basedir, config.right(), localStateDir);
    for (const BlockId& blockId : accountedBlocks) {
        auto num_erased = unaccountedBlocks.erase(blockId);
        ASSERT(1 == num_erased, "Blob id referenced by directory entry but didn't found it on disk? This can't happen.");
    }

    console->print("Calculate statistics\n");

    auto blockStore = makeBlockStore(basedir, config.right(), localStateDir);
    auto nodeStore = make_unique_ref<DataNodeStore>(std::move(blockStore), config.right().configFile->config()->BlocksizeBytes());

    uint32_t numUnaccountedBlocks = unaccountedBlocks.size();
    uint32_t numLeaves = 0;
    uint32_t numInner = 0;
    console->print("Unaccounted blocks: " + std::to_string(unaccountedBlocks.size()) + "\n");
    for (const auto &blockId : unaccountedBlocks) {
        console->print("\r" + std::to_string(numLeaves+numInner) + "/" + std::to_string(numUnaccountedBlocks) + ": ");
        auto node = nodeStore->load(blockId);
        auto innerNode = dynamic_pointer_move<DataInnerNode>(*node);
        if (innerNode != none) {
            ++numInner;
            printNode(std::move(*innerNode));
        }
        auto leafNode = dynamic_pointer_move<DataLeafNode>(*node);
        if (leafNode != none) {
            ++numLeaves;
            printNode(std::move(*leafNode));
        }
    }
    console->print("\n" + std::to_string(numLeaves) + " leaves and " + std::to_string(numInner) + " inner nodes\n");
}
