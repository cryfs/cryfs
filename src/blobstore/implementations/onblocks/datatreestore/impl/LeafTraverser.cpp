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

            void LeafTraverser::traverse(DataNode *root, uint32_t beginIndex, uint32_t endIndex, function<void (uint32_t index, DataLeafNode* leaf)> onExistingLeaf, function<Data (uint32_t index)> onCreateLeaf) {
                ASSERT(beginIndex <= endIndex, "Invalid parameters");

                //TODO Test cases with numLeaves < / >= beginIndex, ideally test all configurations:
                //     beginIndex<endIndex<numLeaves, beginIndex=endIndex<numLeaves, beginIndex<endIndex=numLeaves, beginIndex=endIndex=numLeaves
                //     beginIndex<numLeaves<endIndex, beginIndex=numLeaves<endIndex,
                //     numLeaves<beginIndex<endIndex, numLeaves<beginIndex=endIndex

                _traverseExistingSubtree(root, beginIndex, endIndex, 0, false, onExistingLeaf, onCreateLeaf);
            }

            void LeafTraverser::_traverseExistingSubtree(DataNode *root, uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, bool growLastLeaf, function<void (uint32_t index, DataLeafNode* leaf)> onExistingLeaf, function<Data (uint32_t index)> onCreateLeaf) {
                ASSERT(beginIndex <= endIndex, "Invalid parameters");

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
                uint32_t numChildren = inner->numChildren();
                //ASSERT(!growLastLeaf || endChild == numChildren, "Can only grow last leaf when it is existing");
                bool shouldGrowLastExistingLeaf = growLastLeaf || endChild > numChildren;

                // If we traverse directly outside of the valid region (i.e. usually would only traverse to new leaves and not to the last leaf),
                // we still have to descend to the last leaf to grow it.
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
                                             _createMaxSizeLeaf());
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
                    _traverseExistingSubtree(childNode->get(), localBeginIndex, localEndIndex, leafOffset + childOffset, shouldGrowLastExistingLeaf && isLastChild, onExistingLeaf, onCreateLeaf);
                }

                // Traverse new children (including gap children, i.e. children that are created but not traversed because they're to the right of the current size, but to the left of the traversal region)
                for (uint32_t childIndex = numChildren; childIndex < endChild; ++childIndex) {
                    uint32_t childOffset = childIndex * leavesPerChild;
                    uint32_t localBeginIndex = std::min(leavesPerChild, utils::maxZeroSubtraction(beginIndex, childOffset));
                    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
                    auto leafCreator = (childIndex >= beginChild) ? onCreateLeaf : _createMaxSizeLeaf();
                    auto child = _createNewSubtree(localBeginIndex, localEndIndex, leafOffset + childOffset, inner->depth() - 1, leafCreator);
                    inner->addChild(*child);
                }
            }

            unique_ref<DataNode> LeafTraverser::_createNewSubtree(uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, uint8_t depth, function<Data (uint32_t index)> onCreateLeaf) {
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
                                                   [] (uint32_t /*index*/)->Data {ASSERT(false, "We're only creating gap leaves here, not traversing any.");});
                    children.push_back(std::move(child));
                }
                // Create new children that are traversed
                for(uint32_t childIndex = beginChild; childIndex < endChild; ++childIndex) {
                    uint32_t childOffset = childIndex * leavesPerChild;
                    uint32_t localBeginIndex = utils::maxZeroSubtraction(beginIndex, childOffset);
                    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
                    auto child = _createNewSubtree(localBeginIndex, localEndIndex, leafOffset + childOffset, depth - 1, onCreateLeaf);
                    children.push_back(std::move(child));
                }

                ASSERT(children.size() > 0, "No children created");
                //TODO Performance: Directly create inner node with all children
                auto newNode = _nodeStore->createNewInnerNode(*children[0]);
                for (auto childIter = children.begin()+1; childIter != children.end(); ++childIter) {
                    newNode->addChild(**childIter);
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
