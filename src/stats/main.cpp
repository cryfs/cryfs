#include <iostream>
#include <boost/filesystem.hpp>
#include <cryfs/config/CryConfigLoader.h>
#include <cryfs/config/CryPasswordBasedKeyProvider.h>
#include <blockstore/implementations/ondisk/OnDiskBlockStore2.h>
#include <blockstore/implementations/integrity/IntegrityBlockStore2.h>
#include <blockstore/implementations/low2highlevel/LowToHighLevelBlockStore.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNodeStore.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataInnerNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataLeafNode.h>
#include <blobstore/implementations/onblocks/BlobStoreOnBlocks.h>
#include <cryfs/filesystem/fsblobstore/FsBlobStore.h>
#include <cryfs/filesystem/fsblobstore/DirBlob.h>
#include <cryfs/filesystem/CryDevice.h>
#include <cpp-utils/io/IOStreamConsole.h>
#include <cpp-utils/system/homedir.h>

#include <set>

using namespace boost;
using namespace boost::filesystem;
using namespace std;
using namespace cryfs;
using namespace cpputils;
using namespace blockstore;
using namespace blockstore::ondisk;
using namespace blockstore::integrity;
using namespace blockstore::lowtohighlevel;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;
using namespace cryfs::fsblobstore;

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

void _forEachBlob(FsBlobStore* blobStore, const BlockId& rootId, std::function<void (const BlockId& blobId)> callback) {
    callback(rootId);
    auto rootBlob = blobStore->load(rootId);
    ASSERT(rootBlob != boost::none, "Blob not found but referenced from directory entry");

    auto rootDir = dynamic_pointer_move<DirBlob>(*rootBlob);
    if (rootDir != boost::none) {
        std::vector<fspp::Dir::Entry> children;
        children.reserve((*rootDir)->NumChildren());
        (*rootDir)->AppendChildrenTo(&children);

        for (const auto& child : children) {
            auto childEntry = (*rootDir)->GetChild(child.name);
            ASSERT(childEntry != boost::none, "We just got this from the entry list, it must exist.");
            auto childId = childEntry->blockId();
            _forEachBlob(blobStore, childId, callback);
        }
    }
}

void _forEachBlockInBlob(DataNodeStore* nodeStore, const BlockId& rootId, std::function<void (const BlockId& blockId)> callback) {
    callback(rootId);

    auto node = nodeStore->load(rootId);
    auto innerNode = dynamic_pointer_move<DataInnerNode>(*node);
    if (innerNode != boost::none) {
        for (uint32_t childIndex = 0; childIndex < (*innerNode)->numChildren(); ++childIndex) {
            auto childId = (*innerNode)->readChild(childIndex).blockId();
            _forEachBlockInBlob(nodeStore, childId, callback);
        }
    }
}

unique_ref<BlockStore> makeBlockStore(const path& basedir, const CryConfigLoader::ConfigLoadResult& config, LocalStateDir& localStateDir) {
    auto onDiskBlockStore = make_unique_ref<OnDiskBlockStore2>(basedir);
    auto encryptedBlockStore = CryCiphers::find(config.configFile.config()->Cipher()).createEncryptedBlockstore(std::move(onDiskBlockStore), config.configFile.config()->EncryptionKey());
    auto statePath = localStateDir.forFilesystemId(config.configFile.config()->FilesystemId());
    auto integrityFilePath = statePath / "integritydata";
    auto integrityBlockStore = make_unique_ref<IntegrityBlockStore2>(std::move(encryptedBlockStore), integrityFilePath, config.myClientId, false, true);
    return make_unique_ref<LowToHighLevelBlockStore>(std::move(integrityBlockStore));
}

std::vector<BlockId> _getKnownBlobIds(const path& basedir, const CryConfigLoader::ConfigLoadResult& config, LocalStateDir& localStateDir) {
    auto blockStore = makeBlockStore(basedir, config, localStateDir);
    auto fsBlobStore = make_unique_ref<FsBlobStore>(make_unique_ref<BlobStoreOnBlocks>(std::move(blockStore), config.configFile.config()->BlocksizeBytes()));

    std::vector<BlockId> result;
    cout << "Listing all file system entities (i.e. blobs)..." << flush;
    auto rootId = BlockId::FromString(config.configFile.config()->RootBlob());
    _forEachBlob(fsBlobStore.get(), rootId, [&result] (const BlockId& blockId) {
        result.push_back(blockId);
    });
    cout << "done" << endl;
    return result;
}

std::vector<BlockId> _getKnownBlockIds(const path& basedir, const CryConfigLoader::ConfigLoadResult& config, LocalStateDir& localStateDir) {
    auto knownBlobIds = _getKnownBlobIds(basedir, config, localStateDir);

    auto blockStore = makeBlockStore(basedir, config, localStateDir);
    auto nodeStore = make_unique_ref<DataNodeStore>(std::move(blockStore), config.configFile.config()->BlocksizeBytes());
    std::vector<BlockId> result;
    const uint32_t numNodes = nodeStore->numNodes();
    result.reserve(numNodes);
    uint32_t i = 0;
    cout << "Listing all blocks used by these file system entities..." << endl;
    for (const auto& blobId : knownBlobIds) {
        _forEachBlockInBlob(nodeStore.get(), blobId, [&result, &i, numNodes] (const BlockId& blockId) {
            cout << "\r" << (++i) << "/" << numNodes << flush;
            result.push_back(blockId);
        });
    }
    std::cout << "...done" << endl;
    return result;
}

set<BlockId> _getAllBlockIds(const path& basedir, const CryConfigLoader::ConfigLoadResult& config, LocalStateDir& localStateDir) {
    auto blockStore= makeBlockStore(basedir, config, localStateDir);
    set<BlockId> result;
    blockStore->forEachBlock([&result] (const BlockId& blockId) {
        result.insert(blockId);
    });
    return result;
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

    auto config = config_loader.load(config_path, false, true);
    if (config == boost::none) {
        std::cerr << "Error loading config file" << std::endl;
        exit(1);
    }

    cout << "Listing all blocks..." << flush;
    set<BlockId> unaccountedBlocks = _getAllBlockIds(basedir, *config, localStateDir);
    cout << "done" << endl;

    vector<BlockId> accountedBlocks = _getKnownBlockIds(basedir, *config, localStateDir);
    for (const BlockId& blockId : accountedBlocks) {
        auto num_erased = unaccountedBlocks.erase(blockId);
        ASSERT(1 == num_erased, "Blob id referenced by directory entry but didn't found it on disk? This can't happen.");
    }

    console->print("Calculate statistics\n");

    auto blockStore = makeBlockStore(basedir, *config, localStateDir);
    auto nodeStore = make_unique_ref<DataNodeStore>(std::move(blockStore), config->configFile.config()->BlocksizeBytes());

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
