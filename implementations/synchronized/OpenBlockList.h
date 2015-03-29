#ifndef MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_SYNCHRONIZED_OPENBLOCKLIST_H_
#define MESSMER_BLOCKSTORE_TEST_IMPLEMENTATIONS_SYNCHRONIZED_OPENBLOCKLIST_H_

#include <memory>
#include <set>
#include <map>
#include <vector>
#include <functional>

#include "../../utils/Key.h"
#include <future>

namespace blockstore {
class Block;
namespace synchronized {

class OpenBlockList {
public:
  OpenBlockList();
  virtual ~OpenBlockList();

  std::unique_ptr<Block> insert(std::unique_ptr<Block> block);
  std::unique_ptr<Block> acquire(const Key &key, std::function<std::unique_ptr<Block> ()> loader);

  void release(std::unique_ptr<Block> block);
  void close(std::unique_ptr<Block> block, std::function<void (std::unique_ptr<Block>)> onClose);

private:
  std::set<Key> _openBlocks;
  std::map<Key, std::promise<std::unique_ptr<Block>>> _wantedBlocks;
  std::map<Key, std::promise<std::unique_ptr<Block>>> _blocksToClose;

  std::future<std::unique_ptr<Block>> _addPromiseForBlock(const Key &key);
};

}
}

#endif
