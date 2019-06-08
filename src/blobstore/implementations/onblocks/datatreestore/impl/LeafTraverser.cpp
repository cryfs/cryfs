#include "LeafTraverser.h"
#include <cpp-utils/assert/assert.h>
#include "../../datanodestore/DataLeafNode.h"
#include "../../datanodestore/DataInnerNode.h"
#include "../../datanodestore/DataNodeStore.h"
#include "../../utils/Math.h"

using std::function;
using std::vector;
using boost::none;
using cpputils::Data;
using cpputils::unique_ref;
using cpputils::dynamic_pointer_move;
using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;

namespace blobstore {
    namespace onblocks {
        namespace datatreestore {

            LeafTraverser::LeafTraverser(DataNodeStore *nodeStore, bool readOnlyTraversal)
                : _nodeStore(nodeStore), _readOnlyTraversal(readOnlyTraversal) {
            }

            void LeafTraverser::traverseAndUpdateRoot(unique_ref<DataNode>* root, uint32_t beginIndex, uint32_t endIndex, function<void (uint32_t index, bool isRightBorderLeaf, LeafHandle leaf)> onExistingLeaf, function<Data (uint32_t index)> onCreateLeaf, function<void (DataInnerNode *node)> onBacktrackFromSubtree) {
                _traverseAndUpdateRoot(root, beginIndex, endIndex, true, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);
            }

            void LeafTraverser::_traverseAndUpdateRoot(unique_ref<DataNode>* root, uint32_t beginIndex, uint32_t endIndex, bool isLeftBorderOfTraversal, function<void (uint32_t index, bool isRightBorderLeaf, LeafHandle leaf)> onExistingLeaf, function<Data (uint32_t index)> onCreateLeaf, function<void (DataInnerNode *node)> onBacktrackFromSubtree) {
                ASSERT(beginIndex <= endIndex, "Invalid parameters");

                //TODO Test cases with numLeaves < / >= beginIndex, ideally test all configurations:
                //     beginIndex<endIndex<numLeaves, beginIndex=endIndex<numLeaves, beginIndex<endIndex=numLeaves, beginIndex=endIndex=numLeaves
                //     beginIndex<numLeaves<endIndex, beginIndex=numLeaves<endIndex,
                //     numLeaves<beginIndex<endIndex, numLeaves<beginIndex=endIndex

                uint32_t maxLeavesForDepth = _maxLeavesForTreeDepth((*root)->depth());
                bool increaseTreeDepth = endIndex > maxLeavesForDepth;
                ASSERT(!_readOnlyTraversal || !increaseTreeDepth, "Tried to grow a tree on a read only traversal");

                if ((*root)->depth() == 0) {
                    DataLeafNode *leaf = dynamic_cast<DataLeafNode*>(root->get());
                    ASSERT(leaf != nullptr, "Depth 0 has to be leaf node");

                    if (increaseTreeDepth && leaf->numBytes() != _nodeStore->layout().maxBytesPerLeaf()) {
                        leaf->resize(_nodeStore->layout().maxBytesPerLeaf());
                    }
                    if (beginIndex == 0 && endIndex >= 1) {
                        bool isRightBorderLeaf = (endIndex == 1);
                        onExistingLeaf(0, isRightBorderLeaf, LeafHandle(_nodeStore, leaf));
                    }
                } else {
                    DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root->get());
                    ASSERT(inner != nullptr, "Depth != 0 has to be leaf node");
                    _traverseExistingSubtree(inner, std::min(beginIndex, maxLeavesForDepth),
                                             std::min(endIndex, maxLeavesForDepth), 0, isLeftBorderOfTraversal, !increaseTreeDepth,
                                             increaseTreeDepth, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);
                }

                // If the traversal goes too far right for a tree this depth, increase tree depth by one and continue traversal.
                // This is recursive, i.e. will be repeated if the tree is still not deep enough.
                // We don't increase to the full needed tree depth in one step, because we want the traversal to go as far as possible
                // and only then increase the depth - this causes the tree to be in consistent shape (balanced) for longer.
                if (increaseTreeDepth) {
                    ASSERT(!_readOnlyTraversal, "Can't increase tree depth in a read-only traversal");

                    // TODO Test cases that increase tree depth by 0, 1, 2, ... levels
                    *root = _increaseTreeDepth(std::move(*root));
                    _traverseAndUpdateRoot(root, std::max(beginIndex, maxLeavesForDepth), endIndex, false, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);
                } else {
                    // Once we're done growing the tree and done with the traversal, we might have to decrease tree depth,
                    // because the callbacks could have deleted nodes (this happens for example when shrinking the tree using a traversal).
                    _whileRootHasOnlyOneChildReplaceRootWithItsChild(root);
                }
            }

            unique_ref<DataInnerNode> LeafTraverser::_increaseTreeDepth(unique_ref<DataNode> root) {
                ASSERT(!_readOnlyTraversal, "Can't increase tree depth in a read-only traversal");

                auto copyOfOldRoot = _nodeStore->createNewNodeAsCopyFrom(*root);
                return DataNode::convertToNewInnerNode(std::move(root), _nodeStore->layout(), *copyOfOldRoot);
            }

            void LeafTraverser::_traverseExistingSubtree(const blockstore::BlockId &blockId, uint8_t depth, uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, bool isLeftBorderOfTraversal, bool isRightBorderNode, bool growLastLeaf, function<void (uint32_t index, bool isRightBorderLeaf, LeafHandle leaf)> onExistingLeaf, function<Data (uint32_t index)> onCreateLeaf, function<void (DataInnerNode *node)> onBacktrackFromSubtree) {
                if (depth == 0) {
                    ASSERT(beginIndex <= 1 && endIndex <= 1,
                           "If root node is a leaf, the (sub)tree has only one leaf - access indices must be 0 or 1.");
                    LeafHandle leafHandle(_nodeStore, blockId);
                    if (growLastLeaf) {
                        if (leafHandle.node()->numBytes() != _nodeStore->layout().maxBytesPerLeaf()) {
                            ASSERT(!_readOnlyTraversal, "Can't grow the last leaf in a read-only traversal");
                            leafHandle.node()->resize(_nodeStore->layout().maxBytesPerLeaf());
                        }
                    }
                    if (beginIndex == 0 && endIndex == 1) {
                        onExistingLeaf(leafOffset, isRightBorderNode, std::move(leafHandle));
                    }
                } else {
                    auto node = _nodeStore->load(blockId);
                    if (node == none) {
                        throw std::runtime_error("Couldn't find child node " + blockId.ToString());
                    }

                    auto inner = dynamic_pointer_move<DataInnerNode>(*node);
                    ASSERT(inner != none, "Has to be either leaf or inner node");
                    ASSERT((*inner)->depth() == depth, "Wrong depth given");
                    _traverseExistingSubtree(inner->get(), beginIndex, endIndex, leafOffset, isLeftBorderOfTraversal,
                                             isRightBorderNode, growLastLeaf, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);
                }
            }

            void LeafTraverser::_traverseExistingSubtree(DataInnerNode *root, uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, bool isLeftBorderOfTraversal, bool isRightBorderNode, bool growLastLeaf, function<void (uint32_t index, bool isRightBorderLeaf, LeafHandle leaf)> onExistingLeaf, function<Data (uint32_t index)> onCreateLeaf, function<void (DataInnerNode *node)> onBacktrackFromSubtree) {
                ASSERT(beginIndex <= endIndex, "Invalid parameters");

                //TODO Call callbacks for different leaves in parallel.

                uint32_t leavesPerChild = _maxLeavesForTreeDepth(root->depth()-1);
                uint32_t beginChild = beginIndex/leavesPerChild;
                uint32_t endChild = utils::ceilDivision(endIndex, leavesPerChild);
                ASSERT(endChild <= _nodeStore->layout().maxChildrenPerInnerNode(), "Traversal region would need increasing the tree depth. This should have happened before calling this function.");
                uint32_t numChildren = root->numChildren();
                ASSERT(!growLastLeaf || endChild >= numChildren, "Can only grow last leaf if it exists");
                ASSERT(!_readOnlyTraversal || endChild <= numChildren, "Can only traverse out of bounds in a read-only traversal");
                bool shouldGrowLastExistingLeaf = growLastLeaf || endChild > numChildren;

                // If we traverse outside of the valid region (i.e. usually would only traverse to new leaves and not to the last leaf),
                // we still have to descend to the last old child to fill it with leaves and grow the last old leaf.
                if (isLeftBorderOfTraversal && beginChild >= numChildren) {
                    ASSERT(numChildren > 0, "Node doesn't have children.");
                    auto childBlockId = root->readLastChild().blockId();
                    uint32_t childOffset = (numChildren-1) * leavesPerChild;
                    _traverseExistingSubtree(childBlockId, root->depth()-1, leavesPerChild, leavesPerChild, childOffset, true, false, true,
                                             [] (uint32_t /*index*/, bool /*isRightBorderNode*/, LeafHandle /*leaf*/) {ASSERT(false, "We don't actually traverse any leaves.");},
                                             [] (uint32_t /*index*/) -> Data {ASSERT(false, "We don't actually traverse any leaves.");},
                                             [] (DataInnerNode* /*node*/) {ASSERT(false, "We don't actually traverse any leaves.");});
                }

                // Traverse existing children
                for (uint32_t childIndex = beginChild; childIndex < std::min(endChild, numChildren); ++childIndex) {
                    auto childBlockId = root->readChild(childIndex).blockId();
                    uint32_t childOffset = childIndex * leavesPerChild;
                    uint32_t localBeginIndex = utils::maxZeroSubtraction(beginIndex, childOffset);
                    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
                    bool isFirstChild = (childIndex == beginChild);
                    bool isLastExistingChild = (childIndex == numChildren - 1);
                    bool isLastChild = isLastExistingChild && (numChildren == endChild);
                    ASSERT(localEndIndex <= leavesPerChild, "We don't want the child to add a tree level because it doesn't have enough space for the traversal.");
                    _traverseExistingSubtree(childBlockId, root->depth()-1, localBeginIndex, localEndIndex, leafOffset + childOffset, isLeftBorderOfTraversal && isFirstChild,
                                             isRightBorderNode && isLastChild, shouldGrowLastExistingLeaf && isLastExistingChild, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);
                }

                // Traverse new children (including gap children, i.e. children that are created but not traversed because they're to the right of the current size, but to the left of the traversal region)
                for (uint32_t childIndex = numChildren; childIndex < endChild; ++childIndex) {
                    ASSERT(!_readOnlyTraversal, "Can't create new children in a read-only traversal");

                    uint32_t childOffset = childIndex * leavesPerChild;
                    uint32_t localBeginIndex = std::min(leavesPerChild, utils::maxZeroSubtraction(beginIndex, childOffset));
                    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
                    auto leafCreator = (childIndex >= beginChild) ? onCreateLeaf : _createMaxSizeLeaf();
                    auto child = _createNewSubtree(localBeginIndex, localEndIndex, leafOffset + childOffset, root->depth() - 1, leafCreator, onBacktrackFromSubtree);
                    root->addChild(*child);
                }

                // This is only a backtrack, if we actually visited a leaf here.
                if (endIndex > beginIndex) {
                    onBacktrackFromSubtree(root);
                }
            }

            unique_ref<DataNode> LeafTraverser::_createNewSubtree(uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, uint8_t depth, function<Data (uint32_t index)> onCreateLeaf, function<void (DataInnerNode *node)> onBacktrackFromSubtree) {
                ASSERT(!_readOnlyTraversal, "Can't create a new subtree in a read-only traversal");

                ASSERT(beginIndex <= endIndex, "Invalid parameters");
                if (0 == depth) {
                    ASSERT(beginIndex <= 1 && endIndex == 1, "With depth 0, we can only traverse one or zero leaves (i.e. traverse one leaf or traverse a gap leaf).");
                    auto leafCreator = (beginIndex
                     == 0) ? onCreateLeaf : _createMaxSizeLeaf();
                    return _nodeStore->createNewLeafNode(leafCreator(leafOffset));
                }

                uint8_t minNeededDepth = utils::ceilLog(_nodeStore->layout().maxChildrenPerInnerNode(), static_cast<uint64_t>(endIndex));
                ASSERT(depth >= minNeededDepth, "Given tree depth doesn't fit given number of leaves to create.");
                uint32_t leavesPerChild = _maxLeavesForTreeDepth(depth-1);
                uint32_t beginChild = beginIndex/leavesPerChild;
                uint32_t endChild = utils::ceilDivision(endIndex, leavesPerChild);

                vector<blockstore::BlockId> children;
                children.reserve(endChild);
                // TODO Remove redundancy of following two for loops by using min/max for calculating the parameters of the recursive call.
                // Create gap children (i.e. children before the traversal but after the current size)
                for (uint32_t childIndex = 0; childIndex < beginChild; ++childIndex) {
                    uint32_t childOffset = childIndex * leavesPerChild;
                    auto child = _createNewSubtree(leavesPerChild, leavesPerChild, leafOffset + childOffset, depth - 1,
                                                   [] (uint32_t /*index*/)->Data {ASSERT(false, "We're only creating gap leaves here, not traversing any.");},
                                                   [] (DataInnerNode* /*node*/) {});
                    ASSERT(child->depth() == depth-1, "Created child node has wrong depth");
                    children.push_back(child->blockId());
                }
                // Create new children that are traversed
                for(uint32_t childIndex = beginChild; childIndex < endChild; ++childIndex) {
                    uint32_t childOffset = childIndex * leavesPerChild;
                    uint32_t localBeginIndex = utils::maxZeroSubtraction(beginIndex, childOffset);
                    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
                    auto child = _createNewSubtree(localBeginIndex, localEndIndex, leafOffset + childOffset, depth - 1, onCreateLeaf, onBacktrackFromSubtree);
                    ASSERT(child->depth() == depth-1, "Created child node has wrong depth");
                    children.push_back(child->blockId());
                }

                ASSERT(children.size() > 0, "No children created");
                auto newNode = _nodeStore->createNewInnerNode(depth, children);

                // This is only a backtrack, if we actually created a leaf here.
                if (endIndex > beginIndex) {
                    onBacktrackFromSubtree(newNode.get());
                }
                return newNode;
            }

            uint32_t LeafTraverser::_maxLeavesForTreeDepth(uint8_t depth) const {
                return utils::intPow(_nodeStore->layout().maxChildrenPerInnerNode(), static_cast<uint64_t>(depth));
            }

            function<Data (uint32_t index)> LeafTraverser::_createMaxSizeLeaf() const {
                ASSERT(!_readOnlyTraversal, "Can't create a new leaf in a read-only traversal");

                uint64_t maxBytesPerLeaf = _nodeStore->layout().maxBytesPerLeaf();
                return [maxBytesPerLeaf] (uint32_t /*index*/) -> Data {
                   return Data(maxBytesPerLeaf).FillWithZeroes();
                };
            }

            void LeafTraverser::_whileRootHasOnlyOneChildReplaceRootWithItsChild(unique_ref<DataNode>* root) {
                DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root->get());
                if (inner != nullptr && inner->numChildren() == 1) {
                    ASSERT(!_readOnlyTraversal, "Can't decrease tree depth in a read-only traversal");

                    auto newRoot = _whileRootHasOnlyOneChildRemoveRootReturnChild(inner->readChild(0).blockId());
                    *root = _nodeStore->overwriteNodeWith(std::move(*root), *newRoot);
                    _nodeStore->remove(std::move(newRoot));
                }
            }

            unique_ref<DataNode> LeafTraverser::_whileRootHasOnlyOneChildRemoveRootReturnChild(const blockstore::BlockId &blockId) {
                ASSERT(!_readOnlyTraversal, "Can't decrease tree depth in a read-only traversal");

                auto current = _nodeStore->load(blockId);
                ASSERT(current != none, "Node not found");
                auto inner = dynamic_pointer_move<DataInnerNode>(*current);
                if (inner == none) {
                    return std::move(*current);
                } else if ((*inner)->numChildren() == 1) {
                    auto result = _whileRootHasOnlyOneChildRemoveRootReturnChild((*inner)->readChild(0).blockId());
                    _nodeStore->remove(std::move(*inner));
                    return result;
                } else {
                    return std::move(*inner);
                }
            }

        }
    }
}
