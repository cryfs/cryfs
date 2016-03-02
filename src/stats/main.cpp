#include <iostream>
#include <boost/filesystem.hpp>
#include "../impl/config/CryConfigFile.h"
#include <blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNodeStore.h>
#include <blobstore/implementations/onblocks/datanodestore/DataNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataInnerNode.h>
#include <blobstore/implementations/onblocks/datanodestore/DataLeafNode.h>
#include <blobstore/implementations/onblocks/BlobStoreOnBlocks.h>
#include <cryfs/impl/filesystem/fsblobstore/FsBlobStore.h>
#include <cryfs/impl/filesystem/fsblobstore/DirBlob.h>
#include <cryfs/impl/filesystem/CryDevice.h>

using namespace boost;
using namespace boost::filesystem;
using namespace std;
using namespace cryfs;
using namespace cpputils;
using namespace blockstore;
using namespace blockstore::ondisk;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;
using namespace cryfs::fsblobstore;

void printNode(unique_ref<DataNode> node) {
    std::cout << "Key: " << node->key().ToString() << ", Depth: " << node->depth() << " ";
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

set<Key> _getBlockstoreUnaccountedBlocks(const CryConfig &config) {
    auto onDiskBlockStore = make_unique_ref<OnDiskBlockStore>("/home/heinzi/basedir");
    auto encryptedBlockStore = CryCiphers::find(config.Cipher()).createEncryptedBlockstore(std::move(onDiskBlockStore), config.EncryptionKey());
    auto nodeStore = make_unique_ref<DataNodeStore>(std::move(encryptedBlockStore), CryDevice::BLOCKSIZE_BYTES);
    std::set<Key> unaccountedBlocks;
    uint32_t numBlocks = nodeStore->numNodes();
    uint32_t i = 0;
    cout << "There are " << nodeStore->numNodes() << " blocks." << std::endl;
    // Add all blocks to unaccountedBlocks
    for (auto file = directory_iterator("/home/heinzi/basedir"); file != directory_iterator(); ++file) {
        cout << "\r" << (++i) << "/" << numBlocks << flush;
        if (file->path().filename() != "cryfs.config") {
            auto key = Key::FromString(file->path().filename().c_str());
            unaccountedBlocks.insert(key);
        }
    }
    i = 0;
    cout << "\nRemove blocks that have a parent" << endl;
    //Remove root block from unaccountedBlocks
    unaccountedBlocks.erase(Key::FromString(config.RootBlob()));
    //Remove all blocks that have a parent node from unaccountedBlocks
    for (auto file = directory_iterator("/home/heinzi/basedir"); file != directory_iterator(); ++file) {
        cout << "\r" << (++i) << "/" << numBlocks << flush;
        if (file->path().filename() != "cryfs.config") {
            auto key = Key::FromString(file->path().filename().c_str());
            auto node = nodeStore->load(key);
            auto innerNode = dynamic_pointer_move<DataInnerNode>(*node);
            if (innerNode != none) {
                for (uint32_t childIndex = 0; childIndex < (*innerNode)->numChildren(); ++childIndex) {
                    auto child = (*innerNode)->getChild(childIndex)->key();
                    unaccountedBlocks.erase(child);
                }
            }
        }
    }
    return unaccountedBlocks;
}

set<Key> _getBlocksReferencedByDirEntries(const CryConfig &config) {
    auto onDiskBlockStore = make_unique_ref<OnDiskBlockStore>("/home/heinzi/basedir");
    auto encryptedBlockStore = CryCiphers::find(config.Cipher()).createEncryptedBlockstore(std::move(onDiskBlockStore), config.EncryptionKey());
    auto fsBlobStore = make_unique_ref<FsBlobStore>(make_unique_ref<BlobStoreOnBlocks>(std::move(encryptedBlockStore), CryDevice::BLOCKSIZE_BYTES));
    set<Key> blocksReferencedByDirEntries;
    uint32_t numBlocks = fsBlobStore->numBlocks();
    uint32_t i = 0;
    cout << "\nRemove blocks referenced by dir entries" << endl;
    for (auto file = directory_iterator("/home/heinzi/basedir"); file != directory_iterator(); ++file) {
        cout << "\r" << (++i) << "/" << numBlocks << flush;
        if (file->path().filename() != "cryfs.config") {
            auto key = Key::FromString(file->path().filename().c_str());
            try {
                auto blob = fsBlobStore->load(key);
                if (blob != none) {
                    auto dir = dynamic_pointer_move<DirBlob>(*blob);
                    if (dir != none) {
                        vector<fspp::Dir::Entry> children;
                        (*dir)->AppendChildrenTo(&children);
                        for (const auto &child : children) {
                            blocksReferencedByDirEntries.insert((*dir)->GetChild(child.name)->key);
                        }
                    }
                }
            } catch (...) {}
        }
    }
    return blocksReferencedByDirEntries;
}


int main() {
    cout << "Password: ";
    string password;
    getline(cin, password);
    cout << "Loading config" << endl;
    auto config = CryConfigFile::load("/home/heinzi/basedir/cryfs.config", password);
    set<Key> unaccountedBlocks = _getBlockstoreUnaccountedBlocks(*config->config());
    //Remove all blocks that are referenced by a directory entry from unaccountedBlocks
    set<Key> blocksReferencedByDirEntries = _getBlocksReferencedByDirEntries(*config->config());
    for (const auto &key : blocksReferencedByDirEntries) {
        unaccountedBlocks.erase(key);
    }

    cout << "\nCalculate statistics" << endl;

    auto onDiskBlockStore = make_unique_ref<OnDiskBlockStore>("/home/heinzi/basedir");
    auto encryptedBlockStore = CryCiphers::find(config->config()->Cipher()).createEncryptedBlockstore(std::move(onDiskBlockStore), config->config()->EncryptionKey());
    auto nodeStore = make_unique_ref<DataNodeStore>(std::move(encryptedBlockStore), CryDevice::BLOCKSIZE_BYTES);

    uint32_t numUnaccountedBlocks = unaccountedBlocks.size();
    uint32_t numLeaves = 0;
    uint32_t numInner = 0;
    cout << "\nUnaccounted blocks: " << unaccountedBlocks.size() << endl;
    for (const auto &key : unaccountedBlocks) {
        std::cout << "\r" << (numLeaves+numInner) << "/" << numUnaccountedBlocks << flush;
        auto node = nodeStore->load(key);
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
    cout << "\n" << numLeaves << " leaves and " << numInner << " inner nodes" << endl;
}