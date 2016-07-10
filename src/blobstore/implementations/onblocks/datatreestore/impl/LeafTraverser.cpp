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

                _traverseExistingSubtree(root, beginIndex, endIndex, 0, false, onExistingLeaf, onCreateLeaf);
            }

            void LeafTraverser::_traverseExistingSubtree(DataNode *root, uint32_t beginIndex, uint32_t endIndex, uint32_t leafOffset, bool growLastExistingLeaf, function<void (uint32_t index, DataLeafNode* leaf)> onExistingLeaf, function<Data (uint32_t index)> onCreateLeaf) {
                ASSERT(beginIndex <= endIndex, "Invalid parameters");

                //TODO Test cases with numLeaves < / >= beginIndex

                DataLeafNode *leaf = dynamic_cast<DataLeafNode*>(root);
                if (leaf != nullptr) {
                    ASSERT(beginIndex <= 1 && endIndex <= 1, "If root node is a leaf, the (sub)tree has only one leaf - access indices must be 0 or 1.");
                    if (beginIndex == 0 && endIndex == 1) {
                        if (growLastExistingLeaf) {
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
                bool shouldGrowLastExistingLeaf = growLastExistingLeaf || endChild > numChildren;

                // If we traverse outside of the valid region, we still have to descend into the valid region to grow the last leaf
                if (beginChild >= numChildren && shouldGrowLastExistingLeaf) {
                    ASSERT(beginChild > 0, "This can only happen for numChildren==0.");
                    auto childKey = inner->getChild(beginChild-1)->key();
                    auto childNode = _nodeStore->load(childKey);
                    if (childNode == none) {
                        throw std::runtime_error("Couldn't find child node "+childKey.ToString());
                    }
                    _traverseExistingSubtree(childNode->get(), leavesPerChild-1, leavesPerChild, leafOffset - 1, true,
                                             [] (uint32_t /*index*/, DataLeafNode* /*leaf*/) {},
                                             [] (uint32_t /*index*/) -> Data {ASSERT(false, "We only want to grow the last leaf. We shouldn't create leaves.");});
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

                // Traverse gap children (children after currently last leaf that are not traversed, i.e. before the first traversed leaf)
                for (uint32_t childIndex = numChildren; childIndex < endChild; ++childIndex) {
                    uint32_t childOffset = childIndex * leavesPerChild;
                    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
                    ASSERT(beginIndex <= childOffset, "Range for creating new children has to contain their first leaf.");
                    uint64_t maxBytesPerLeaf = _nodeStore->layout().maxBytesPerLeaf();
                    auto child = _createNewSubtree(localEndIndex, leafOffset + childOffset, inner->depth()-1, [maxBytesPerLeaf] (uint32_t /*index*/) {
                        return Data(maxBytesPerLeaf).FillWithZeroes();
                    });
                    inner->addChild(*child);
                }

                // Traverse new children (children after currently last leaf that are traversed)
                for (uint32_t childIndex = std::max(beginChild, numChildren); childIndex < endChild; ++childIndex) {
                    uint32_t childOffset = childIndex * leavesPerChild;
                    uint32_t localEndIndex = std::min(leavesPerChild, endIndex - childOffset);
                    ASSERT(beginIndex <= childOffset, "Range for creating new children has to contain their first leaf.");
                    auto child = _createNewSubtree(localEndIndex, leafOffset + childOffset, inner->depth()-1, onCreateLeaf);
                    inner->addChild(*child);
                }
            }

            unique_ref<DataNode> LeafTraverser::_createNewSubtree(uint32_t numLeaves, uint32_t leafOffset, uint8_t depth, function<Data (uint32_t index)> onCreateLeaf) {
                ASSERT(depth > 0, "Wrong depth given");
                if (1 == depth) {
                    ASSERT(numLeaves == 1, "With depth 1, we can only create one leaf.");
                    auto data = onCreateLeaf(leafOffset);
                    // TODO Performance: Directly create with data.
                    auto node = _nodeStore->createNewLeafNode();
                    node->resize(data.size());
                    node->write(data.data(), 0, data.size());
                    return node;
                }

                uint8_t minNeededDepth = utils::ceilLog(_nodeStore->layout().maxChildrenPerInnerNode(), (uint64_t)numLeaves);
                ASSERT(depth >= minNeededDepth, "Given tree depth doesn't fit given number of leaves to create.");
                uint32_t leavesPerChild = _maxLeavesForTreeDepth(depth-1);
                uint32_t endChild = utils::ceilDivision(numLeaves, leavesPerChild);

                vector<unique_ref<DataNode>> children;
                children.reserve(endChild);
                for(uint32_t childIndex = 0; childIndex < endChild; ++childIndex) {
                    uint32_t childOffset = childIndex * leavesPerChild;
                    uint32_t localNumLeaves = std::min(leavesPerChild, numLeaves - childOffset);
                    auto child = _createNewSubtree(localNumLeaves, leafOffset + childOffset, depth - 1, onCreateLeaf);
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

            uint32_t LeafTraverser::_maxLeavesForTreeDepth(uint8_t depth) {
                return utils::intPow(_nodeStore->layout().maxChildrenPerInnerNode(), (uint64_t)depth);
            }

        }
    }
}
