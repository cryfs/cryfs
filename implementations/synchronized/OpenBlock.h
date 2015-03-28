#pragma once
#ifndef BLOCKSTORE_IMPLEMENTATIONS_SYNCHRONIZED_OPENBLOCK_H_
#define BLOCKSTORE_IMPLEMENTATIONS_SYNCHRONIZED_OPENBLOCK_H_

#include "../../interface/Block.h"

#include "messmer/cpp-utils/macros.h"
#include <memory>

namespace blockstore {
namespace synchronized {
class OpenBlockList;

class OpenBlock: public Block {
public:
  OpenBlock(std::unique_ptr<Block> baseBlock, OpenBlockList *openBlockList);
  virtual ~OpenBlock();

  const void *data() const override;
  void write(const void *source, uint64_t offset, uint64_t size) override;

  void flush() override;

  size_t size() const override;

private:
  std::unique_ptr<Block> _baseBlock;
  OpenBlockList *_openBlockList;

  DISALLOW_COPY_AND_ASSIGN(OpenBlock);
};

}
}

#endif
