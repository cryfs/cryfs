#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_LEAFHANDLE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_LEAFHANDLE_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/optional_ownership_ptr.h>
#include "blobstore/implementations/onblocks/datanodestore/DataNodeStore.h"
#include "blobstore/implementations/onblocks/datanodestore/DataLeafNode.h"

namespace blobstore {
    namespace onblocks {
        namespace datatreestore {

            class LeafHandle final {
            public:
                LeafHandle(datanodestore::DataNodeStore *nodeStore, const blockstore::Key &key)
                    :_nodeStore(nodeStore), _key(key), _leaf(cpputils::null<datanodestore::DataLeafNode>()) {
                }

                LeafHandle(datanodestore::DataNodeStore *nodeStore, datanodestore::DataLeafNode *node)
                        : _nodeStore(nodeStore), _key(node->key()), _leaf(cpputils::WithoutOwnership<datanodestore::DataLeafNode>(node)) {
                }

                LeafHandle(LeafHandle &&rhs) = default;

                const blockstore::Key &key() {
                    return _key;
                }

                datanodestore::DataLeafNode *node() {
                    if (_leaf.get() == nullptr) {
                        auto loaded = _nodeStore->load(_key);
                        ASSERT(loaded != boost::none, "Leaf not found");
                        auto leaf = cpputils::dynamic_pointer_move<datanodestore::DataLeafNode>(*loaded);
                        ASSERT(leaf != boost::none, "Loaded leaf is not leaf node");

                        _leaf = cpputils::WithOwnership(std::move(*leaf));
                    }

                    return _leaf.get();
                }

            private:
                datanodestore::DataNodeStore *_nodeStore;
                blockstore::Key _key;
                cpputils::optional_ownership_ptr<datanodestore::DataLeafNode> _leaf;

                DISALLOW_COPY_AND_ASSIGN(LeafHandle);
            };


        }
    }
}

#endif
