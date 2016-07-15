#include "LeafHandle.h"
#include "../datanodestore/DataLeafNode.h"
#include "../datanodestore/DataNodeStore.h"

using cpputils::WithOwnership;
using cpputils::WithoutOwnership;
using boost::none;
using cpputils::dynamic_pointer_move;
using blobstore::onblocks::datanodestore::DataLeafNode;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blockstore::Key;

namespace blobstore {
    namespace onblocks {
        namespace datatreestore {

            LeafHandle::LeafHandle(DataNodeStore *nodeStore, const Key &key)
                    : _nodeStore(nodeStore), _key(key), _leaf(cpputils::null<DataLeafNode>()) {
            }

            LeafHandle::LeafHandle(DataNodeStore *nodeStore, DataLeafNode *node)
                    : _nodeStore(nodeStore), _key(node->key()),
                      _leaf(WithoutOwnership<DataLeafNode>(node)) {
            }

            DataLeafNode *LeafHandle::node() {
                if (_leaf.get() == nullptr) {
                    auto loaded = _nodeStore->load(_key);
                    ASSERT(loaded != none, "Leaf not found");
                    auto leaf = dynamic_pointer_move<DataLeafNode>(*loaded);
                    ASSERT(leaf != none, "Loaded leaf is not leaf node");

                    _leaf = WithOwnership(std::move(*leaf));
                }

                return _leaf.get();
            }
        }
    }
}
