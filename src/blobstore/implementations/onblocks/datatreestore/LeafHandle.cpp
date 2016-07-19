#include "LeafHandle.h"
#include "../datanodestore/DataLeafNode.h"
#include "../datanodestore/DataNodeStore.h"

using cpputils::WithOwnership;
using cpputils::WithoutOwnership;
using boost::none;
using boost::optional;
using cpputils::dynamic_pointer_move;
using cpputils::unique_ref;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blockstore::Key;

namespace blobstore {
    namespace onblocks {
        namespace datatreestore {

            LeafHandle::LeafHandle(DataNodeStore *nodeStore, const Key &key, optional<size_t> size)
                    : _nodeStore(nodeStore), _key(key), _leaf(cpputils::null<DataLeafNode>()), _size(size) {
            }

            LeafHandle::LeafHandle(DataNodeStore *nodeStore, DataLeafNode *node)
                    : _nodeStore(nodeStore), _key(node->key()),
                      _leaf(WithoutOwnership<DataLeafNode>(node)) {
            }

            DataLeafNode *LeafHandle::loadForReading() {
                if (_leaf.get() == nullptr) {
                    auto loaded = _nodeStore->load(_key);
                    ASSERT(loaded != none, "Node not found");
                    auto leaf = dynamic_pointer_move<DataLeafNode>(*loaded);
                    ASSERT(leaf != none, "Node is not a leaf node");
                    _leaf = WithOwnership(std::move(*leaf));
                }

                return _leaf.get();
            }

            DataLeafNode *LeafHandle::loadForWriting() {
                // loadForWriting() is optimized so it doesn't cause the block to be loaded/read from the disk.
                // This is great if we only do write accesses afterwards, because it allows overwriting
                // blocks without ever loading them.
                // But it only works if we know the size. If we don't know the size, we use the standard
                // algorithm loadForReading().
                if (_size == none) {
                    return loadForReading();
                }
                if (_leaf.get() == nullptr) {
                    auto leaf = _nodeStore->loadOrCreateLeaf(_key, *_size);
                    _leaf = WithOwnership(std::move(leaf));
                }

                return _leaf.get();
            }

            unique_ref<DataLeafNode> LeafHandle::_load() {
                // If we know the size of the leaf, we don't have to load it. loadOrCreate() is enough and faster.
                if (_size != none) {
                    return _nodeStore->loadOrCreateLeaf(_key, *_size);
                } else {
                    auto loaded = _nodeStore->load(_key);
                    ASSERT(loaded != none, "Node not found");
                    auto leaf = dynamic_pointer_move<DataLeafNode>(*loaded);
                    ASSERT(leaf != none, "Node is not a leaf node");
                    return std::move(*leaf);
                }

            }
        }
    }
}
