#pragma once
#ifndef MESSMER_PARALLELACCESSSTORE_PARALLELACCESSBASESTORE_H_
#define MESSMER_PARALLELACCESSSTORE_PARALLELACCESSBASESTORE_H_

#include <cpp-utils/pointer/unique_ref.h>
#include <boost/optional.hpp>
#include <blockstore/utils/BlockId.h>

namespace parallelaccessstore {

template<class Resource, class Key>
class ParallelAccessBaseStore {
public:
  virtual ~ParallelAccessBaseStore() {}
  virtual boost::optional<cpputils::unique_ref<Resource>> loadFromBaseStore(const Key &key) = 0;
  virtual void removeFromBaseStore(cpputils::unique_ref<Resource> block) = 0;
  virtual void removeFromBaseStore(const blockstore::BlockId &blockId) = 0;
};

}

#endif
