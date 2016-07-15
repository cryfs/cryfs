#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_LEAFHANDLE_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_LEAFHANDLE_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/optional_ownership_ptr.h>
#include <blockstore/utils/Key.h>

namespace blobstore {
    namespace onblocks {
        namespace datanodestore {
            class DataNodeStore;
            class DataLeafNode;
        }
        namespace datatreestore {

            class LeafHandle final {
            public:
                LeafHandle(datanodestore::DataNodeStore *nodeStore, const blockstore::Key &key);
                LeafHandle(datanodestore::DataNodeStore *nodeStore, datanodestore::DataLeafNode *node);
                LeafHandle(LeafHandle &&rhs) = default;

                const blockstore::Key &key() {
                    return _key;
                }

                datanodestore::DataLeafNode *node();

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
