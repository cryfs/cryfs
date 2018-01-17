#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_LEAFHANDLE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_LEAFHANDLE_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/optional_ownership_ptr.h>
#include <blockstore/utils/BlockId.h>

namespace blobstore {
    namespace onblocks {
        namespace datanodestore {
            class DataNodeStore;
            class DataLeafNode;
        }
        namespace datatreestore {

            class LeafHandle final {
            public:
                LeafHandle(datanodestore::DataNodeStore *nodeStore, const blockstore::BlockId &blockId);
                LeafHandle(datanodestore::DataNodeStore *nodeStore, datanodestore::DataLeafNode *node);
                LeafHandle(LeafHandle &&rhs) = default;

                const blockstore::BlockId &blockId() {
                    return _blockId;
                }

                datanodestore::DataLeafNode *node();

                datanodestore::DataNodeStore *nodeStore() {
                    return _nodeStore;
                }

                bool isLoaded() const {
                    return _leaf.get() != nullptr;
                }

            private:
                datanodestore::DataNodeStore *_nodeStore;
                blockstore::BlockId _blockId;
                cpputils::optional_ownership_ptr<datanodestore::DataLeafNode> _leaf;

                DISALLOW_COPY_AND_ASSIGN(LeafHandle);
            };


        }
    }
}

#endif
