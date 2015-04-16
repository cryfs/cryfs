#ifndef MESSMER_PARALLELACCESSSTORE_PARALLELACCESSBASESTORE_H_
#define MESSMER_PARALLELACCESSSTORE_PARALLELACCESSBASESTORE_H_

#include <memory>

namespace parallelaccessstore {

template<class Resource, class Key>
class ParallelAccessBaseStore {
public:
  virtual ~ParallelAccessBaseStore() {}
  virtual std::unique_ptr<Resource> loadFromBaseStore(const Key &key) = 0;
  virtual void removeFromBaseStore(std::unique_ptr<Resource> block) = 0;
};

}

#endif
