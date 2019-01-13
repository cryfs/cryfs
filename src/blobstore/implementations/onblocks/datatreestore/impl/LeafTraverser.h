#pragma once
#ifndef MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_LEAFTRAVERSER_H_
#define MESSMER_BLOBSTORE_IMPLEMENTATIONS_ONBLOCKS_IMPL_LEAFTRAVERSER_H_

#include <cpp-utils/macros.h>
#include <cpp-utils/pointer/unique_ref.h>
#include <cpp-utils/data/Data.h>
#include <blockstore/utils/BlockId.h>
#include "blobstore/implementations/onblocks/datatreestore/LeafHandle.h"

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
                LeafTraverser(datanodestore::DataNodeStore *nodeStore, bool readOnlyTraversal);

                void traverseAndUpdateRoot(
                      cpputils::unique_ref<datanodestore::DataNode>* root, uint32_t beginIndex, uint32_t endIndex,
                      std::function<void (uint32_t index, bool isRightBorderLeaf, LeafHandle leaf)> onExistingLeaf,
                      std::function<cpputils::Data (uint32_t index)> onCreateLeaf,
                      std::function<void (datanodestore::DataInnerNode *node)> onBacktrackFromSubtree);

            private:
                datanodestore::DataNodeStore *_nodeStore;
                const bool _readOnlyTraversal;

                void _traverseAndUpdateRoot(
                      cpputils::unique_ref<datanodestore::DataNode>* root, uint32_t beginIndex, uint32_t endIndex, bool isLeftBorderOfTraversal,
                      std::function<void (uint32_t index, bool isRightBorderLeaf, LeafHandle leaf)> onExistingLeaf,
                      std::function<cpputils::Data (uint32_t index)> onCreateLeaf,
                      std::function<void (datanodestore::DataInnerNode *node)> onBacktrackFromSubtree);
                void _traverseExistingSubtree(datanodestore::DataInnerNode *root, uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, bool isLeftBorderOfTraversal, bool isRightBorderNode, bool growLastLeaf,
                      std::function<void (uint32_t index, bool isRightBorderLeaf, LeafHandle leaf)> onExistingLeaf,
                      std::function<cpputils::Data (uint32_t index)> onCreateLeaf,
                      std::function<void (datanodestore::DataInnerNode *node)> onBacktrackFromSubtree);
                void _traverseExistingSubtree(const blockstore::BlockId &blockId, uint8_t depth, uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, bool isLeftBorderOfTraversal, bool isRightBorderNode, bool growLastLeaf,
                                              std::function<void (uint32_t index, bool isRightBorderLeaf, LeafHandle leaf)> onExistingLeaf,
                                              std::function<cpputils::Data (uint32_t index)> onCreateLeaf,
                                              std::function<void (datanodestore::DataInnerNode *node)> onBacktrackFromSubtree);
                cpputils::unique_ref<datanodestore::DataInnerNode> _increaseTreeDepth(cpputils::unique_ref<datanodestore::DataNode> root);
                cpputils::unique_ref<datanodestore::DataNode> _createNewSubtree(uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, uint8_t depth,
                                                                                std::function<cpputils::Data (uint32_t index)> onCreateLeaf,
                                                                                std::function<void (datanodestore::DataInnerNode *node)> onBacktrackFromSubtree);
                uint32_t _maxLeavesForTreeDepth(uint8_t depth) const;
                std::function<cpputils::Data (uint32_t index)> _createMaxSizeLeaf() const;
                void _whileRootHasOnlyOneChildReplaceRootWithItsChild(cpputils::unique_ref<datanodestore::DataNode>* root);
                cpputils::unique_ref<datanodestore::DataNode> _whileRootHasOnlyOneChildRemoveRootReturnChild(const blockstore::BlockId &blockId);

                DISALLOW_COPY_AND_ASSIGN(LeafTraverser);
            };

        }
    }
}


#endif
