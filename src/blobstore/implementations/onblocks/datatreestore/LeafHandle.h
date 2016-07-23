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
                LeafHandle(datanodestore::DataNodeStore *nodeStore, const blockstore::Key &key, boost::optional<size_t> size);
                LeafHandle(datanodestore::DataNodeStore *nodeStore, datanodestore::DataLeafNode *node);
                LeafHandle(LeafHandle &&rhs) = default;

                const blockstore::Key &key() {
                    return _key;
                }

                datanodestore::DataLeafNode *loadForReading();
                datanodestore::DataLeafNode *loadForWriting();

                datanodestore::DataNodeStore *nodeStore() {
                    return _nodeStore;
                }

            private:
                datanodestore::DataNodeStore *_nodeStore;
                blockstore::Key _key;
                cpputils::optional_ownership_ptr<datanodestore::DataLeafNode> _leaf;
                boost::optional<size_t> _size; // Stores the size of the leaf if we know it (i.e. it isn't the last leaf)

                DISALLOW_COPY_AND_ASSIGN(LeafHandle);
            };


        }
    }
}

#endif
