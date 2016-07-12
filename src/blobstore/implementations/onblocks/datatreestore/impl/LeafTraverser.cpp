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
using blobstore::onblocks::datanodestore::DataNodeStore;
using blobstore::onblocks::datanodestore::DataNode;
using blobstore::onblocks::datanodestore::DataInnerNode;
using blobstore::onblocks::datanodestore::DataLeafNode;

namespace blobstore {
    namespace onblocks {
        namespace datatreestore {

            LeafTraverser::LeafTraverser(DataNodeStore *nodeStore)
                : _nodeStore(nodeStore) {
            }

            unique_ref<DataNode> LeafTraverser::traverseAndReturnRoot(unique_ref<DataNode> root, uint32_t beginIndex, uint32_t endIndex, function<void (uint32_t index, DataLeafNode* leaf)> onExistingLeaf, function<Data (uint32_t index)> onCreateLeaf, function<void (DataInnerNode *node)> onBacktrackFromSubtree) {
                ASSERT(beginIndex <= endIndex, "Invalid parameters");

                //TODO Test cases with numLeaves < / >= beginIndex, ideally test all configurations:
                //     beginIndex<endIndex<numLeaves, beginIndex=endIndex<numLeaves, beginIndex<endIndex=numLeaves, beginIndex=endIndex=numLeaves
                //     beginIndex<numLeaves<endIndex, beginIndex=numLeaves<endIndex,
                //     numLeaves<beginIndex<endIndex, numLeaves<beginIndex=endIndex

                uint32_t maxLeavesForDepth = _maxLeavesForTreeDepth(root->depth());
                bool increaseTreeDepth = endIndex > maxLeavesForDepth;

                _traverseExistingSubtree(root.get(), std::min(beginIndex, maxLeavesForDepth), std::min(endIndex, maxLeavesForDepth), 0, increaseTreeDepth, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);

                // If the traversal goes too far right for a tree this depth, increase tree depth by one and continue traversal.
                // This is recursive, i.e. will be repeated if the tree is still not deep enough.
                // We don't increase to the full needed tree depth in one step, because we want the traversal to go as far as possible
                // and only then increase the depth - this causes the tree to be in consistent shape (balanced) for longer.
                if (increaseTreeDepth) {
                    // TODO Test cases that increase tree depth by 0, 1, 2, ... levels
                    auto newRoot = _increaseTreeDepth(std::move(root));
                    return traverseAndReturnRoot(std::move(newRoot), std::max(beginIndex, maxLeavesForDepth), endIndex, onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);
                } else {
                    return std::move(root);
                }
            }

            unique_ref<DataInnerNode> LeafTraverser::_increaseTreeDepth(unique_ref<DataNode> root) {
                auto copyOfOldRoot = _nodeStore->createNewNodeAsCopyFrom(*root);
                return DataNode::convertToNewInnerNode(std::move(root), *copyOfOldRoot);
            }

            void LeafTraverser::_traverseExistingSubtree(DataNode *root, uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, bool growLastLeaf, function<void (uint32_t index, DataLeafNode* leaf)> onExistingLeaf, function<Data (uint32_t index)> onCreateLeaf, function<void (DataInnerNode *node)> onBacktrackFromSubtree) {
                ASSERT(beginIndex <= endIndex, "Invalid parameters");

                //TODO Call callbacks for different leaves in parallel.

                DataLeafNode *leaf = dynamic_cast<DataLeafNode*>(root);
                if (leaf != nullptr) {
                    ASSERT(beginIndex <= 1 && endIndex <= 1, "If root node is a leaf, the (sub)tree has only one leaf - access indices must be 0 or 1.");
                    if (beginIndex == 0 && endIndex == 1) {
                        if (growLastLeaf) {
                            leaf->resize(_nodeStore->layout().maxBytesPerLeaf());
                        }
                        onExistingLeaf(leafOffset, leaf);
                    }
                    return;
                }

                DataInnerNode *inner = dynamic_cast<DataInnerNode*>(root);

                uint32_t leavesPerChild = _maxLeavesForTreeDepth(inner->depth()-1);
                uint32_t beginChild = beginIndex/leavesPerChild;
                uint32_t endChild = utils::ceilDivision(endIndex, leavesPerChild);
                ASSERT(endChild <= _nodeStore->layout().maxChildrenPerInnerNode(), "Traversal region would need increasing the tree depth. This should have happened before calling this function.");
                uint32_t numChildren = inner->numChildren();
                bool shouldGrowLastExistingLeaf = growLastLeaf || endChild > numChildren;

                // If we traverse outside of the valid region (i.e. usually would only traverse to new leaves and not to the last leaf),
                // we still have to descend to the last old child to fill it with leaves and grow the last old leaf.
                if (beginChild >= numChildren) {
                    ASSERT(numChildren > 0, "Node doesn't have children.");
                    auto childKey = inner->getChild(numChildren-1)->key();
                    auto childNode = _nodeStore->load(childKey);
                    if (childNode == none) {
                        throw std::runtime_error("Couldn't find child node "+childKey.ToString());
                    }
                    //TODO this causes a negative leafOffset. Better: Make leafOffset generally absolute, i.e. += beginIndex?
                    uint32_t negativeChildOffset = (numChildren-1) * leavesPerChild;
                    _traverseExistingSubtree(childNode->get(), leavesPerChild-1, leavesPerChild, leafOffset - negativeChildOffset, true,
                                             [] (uint32_t /*index*/, DataLeafNode* /*leaf*/) {},
                                             _createMaxSizeLeaf(),
                                             [] (DataInnerNode* /*node*/) {});
                }

                // Traverse existing children
                for (uint32_t childIndex = beginChild; childIndex < std::min(endChild, numChildren); ++childIndex) {
                    auto childKey = inner->getChild(childIndex)->key();
                    auto childNode = _nodeStore->load(childKey);
                    if (childNode == none) {
                        throw std::runtime_error("Couldn't find child node "+childKey.ToString());
                    }
                    uint32_t childOffset = childIndex * leavesPerChild;
                    uint32_t localBeginIndex = utils::maxZeroSubtraction(beginIndex, childOffset);
                    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
                    bool isLastChild = (childIndex == numChildren - 1);
                    ASSERT(localEndIndex <= leavesPerChild, "We don't want the child to add a tree level because it doesn't have enough space for the traversal.");
                    _traverseExistingSubtree(childNode->get(), localBeginIndex, localEndIndex, leafOffset + childOffset, shouldGrowLastExistingLeaf && isLastChild,
                                             onExistingLeaf, onCreateLeaf, onBacktrackFromSubtree);
                }

                // Traverse new children (including gap children, i.e. children that are created but not traversed because they're to the right of the current size, but to the left of the traversal region)
                for (uint32_t childIndex = numChildren; childIndex < endChild; ++childIndex) {
                    uint32_t childOffset = childIndex * leavesPerChild;
                    uint32_t localBeginIndex = std::min(leavesPerChild, utils::maxZeroSubtraction(beginIndex, childOffset));
                    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
                    auto leafCreator = (childIndex >= beginChild) ? onCreateLeaf : _createMaxSizeLeaf();
                    auto child = _createNewSubtree(localBeginIndex, localEndIndex, leafOffset + childOffset, inner->depth() - 1, leafCreator, onBacktrackFromSubtree);
                    inner->addChild(*child);
                }

                // This is only a backtrack, if we actually visited a leaf here.
                if (endIndex > beginIndex) {
                    onBacktrackFromSubtree(inner);
                }
            }

            unique_ref<DataNode> LeafTraverser::_createNewSubtree(uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, uint8_t depth, function<Data (uint32_t index)> onCreateLeaf, function<void (DataInnerNode *node)> onBacktrackFromSubtree) {
                ASSERT(beginIndex <= endIndex, "Invalid parameters");
                if (0 == depth) {
                    ASSERT(beginIndex <= 1 && endIndex == 1, "With depth 0, we can only traverse one or zero leaves (i.e. traverse one leaf or traverse a gap leaf).");
                    auto leafCreator = (beginIndex == 0) ? onCreateLeaf : _createMaxSizeLeaf();
                    auto data = leafCreator(leafOffset);
                    // TODO Performance: Directly create leaf node with data.
                    auto node = _nodeStore->createNewLeafNode();
                    node->resize(data.size());
                    node->write(data.data(), 0, data.size());
                    return node;
                }

                uint8_t minNeededDepth = utils::ceilLog(_nodeStore->layout().maxChildrenPerInnerNode(), (uint64_t)endIndex);
                ASSERT(depth >= minNeededDepth, "Given tree depth doesn't fit given number of leaves to create.");
                uint32_t leavesPerChild = _maxLeavesForTreeDepth(depth-1);
                uint32_t beginChild = beginIndex/leavesPerChild;
                uint32_t endChild = utils::ceilDivision(endIndex, leavesPerChild);

                vector<unique_ref<DataNode>> children;
                children.reserve(endChild);
                // TODO Remove redundancy of following two for loops by using min/max for calculating the parameters of the recursive call.
                // Create gap children (i.e. children before the traversal but after the current size)
                for (uint32_t childIndex = 0; childIndex < beginChild; ++childIndex) {
                    uint32_t childOffset = childIndex * leavesPerChild;
                    auto child = _createNewSubtree(leavesPerChild, leavesPerChild, leafOffset + childOffset, depth - 1,
                                                   [] (uint32_t /*index*/)->Data {ASSERT(false, "We're only creating gap leaves here, not traversing any.");},
                                                   [] (DataInnerNode* /*node*/) {});
                    children.push_back(std::move(child));
                }
                // Create new children that are traversed
                for(uint32_t childIndex = beginChild; childIndex < endChild; ++childIndex) {
                    uint32_t childOffset = childIndex * leavesPerChild;
                    uint32_t localBeginIndex = utils::maxZeroSubtraction(beginIndex, childOffset);
                    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
                    auto child = _createNewSubtree(localBeginIndex, localEndIndex, leafOffset + childOffset, depth - 1, onCreateLeaf, onBacktrackFromSubtree);
                    children.push_back(std::move(child));
                }

                ASSERT(children.size() > 0, "No children created");
                //TODO Performance: Directly create inner node with all children
                auto newNode = _nodeStore->createNewInnerNode(*children[0]);
                for (auto childIter = children.begin()+1; childIter != children.end(); ++childIter) {
                    newNode->addChild(**childIter);
                }
                // This is only a backtrack, if we actually created a leaf here.
                if (endIndex > beginIndex) {
                    onBacktrackFromSubtree(newNode.get());
                }
                return newNode;
            }

            uint32_t LeafTraverser::_maxLeavesForTreeDepth(uint8_t depth) const {
                return utils::intPow(_nodeStore->layout().maxChildrenPerInnerNode(), (uint64_t)depth);
            }

            function<Data (uint32_t index)> LeafTraverser::_createMaxSizeLeaf() const {
                uint64_t maxBytesPerLeaf = _nodeStore->layout().maxBytesPerLeaf();
                return [maxBytesPerLeaf] (uint32_t /*index*/) -> Data {
                   return Data(maxBytesPerLeaf).FillWithZeroes();
                };
            }

        }
    }
}
