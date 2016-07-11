#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_LEAFTRAVERSER_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_LEAFTRAVERSER_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/data/Data.h>

namespace blobstore {
    namespace onblocks {
        namespace datanodestore {
            class DataNodeStore;
            class DataNode;
            class DataLeafNode;
            class DataInnerNode;
        }
        namespace datatreestore {

            /**
             * LeafTraverser can create leaves if they don't exist yet (i.e. endIndex > numLeaves), but
             * it cannot increase the tree depth. That is, the tree has to be deep enough to allow
             * creating the number of leaves.
             */
            class LeafTraverser final {
            public:
                LeafTraverser(datanodestore::DataNodeStore *nodeStore);

                void traverse(datanodestore::DataNode *root, uint32_t beginIndex, uint32_t endIndex, std::function<void (uint32_t index, datanodestore::DataLeafNode* leaf)> onExistingLeaf, std::function<cpputils::Data (uint32_t index)> onCreateLeaf);

            private:
                datanodestore::DataNodeStore *_nodeStore;

                void _traverseExistingSubtree(datanodestore::DataNode *root, uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, bool growLastLeaf, std::function<void (uint32_t index, datanodestore::DataLeafNode* leaf)> onExistingLeaf, std::function<cpputils::Data (uint32_t index)> onCreateLeaf);
                cpputils::unique_ref<datanodestore::DataNode> _createNewSubtree(uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, uint8_t depth, std::function<cpputils::Data (uint32_t index)> onCreateLeaf);
                uint32_t _maxLeavesForTreeDepth(uint8_t depth) const;
                std::function<cpputils::Data (uint32_t index)> _createMaxSizeLeaf() const;

                DISALLOW_COPY_AND_ASSIGN(LeafTraverser);
            };

        }
    }
}


#endif
