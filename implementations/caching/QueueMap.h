#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING2_MAP_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING2_MAP_H_

#include <memory>
#include <unordered_map>
#include <cassert>

namespace blockstore {
namespace caching {

//TODO Test
//TODO Move to utils
template<class Key, class Value>
class QueueMap {
public:
  QueueMap(): _entries(), _sentinel(Key(), nullptr, &_sentinel, &_sentinel) {
  }
  virtual ~QueueMap() {}

  void push(const Key &key, std::unique_ptr<Value> value) {
    auto newEntry = std::make_unique<Entry>(key, std::move(value), _sentinel.prev, &_sentinel);
    _sentinel.prev->next = newEntry.get();
    _sentinel.prev = newEntry.get();
    auto insertResult = _entries.emplace(key, std::move(newEntry));
    assert(insertResult.second == true);
  }

  std::unique_ptr<Value> pop(const Key &key) {
    auto found = _entries.find(key);
    if (found == _entries.end()) {
      return nullptr;
    }
    _removeFromQueue(found->second.get());
    auto value = std::move(found->second->value);
    _entries.erase(found);
    return value;
  }

  std::unique_ptr<Value> pop() {
    return pop(_sentinel.next->key);
  }

  uint32_t size() {
    return _entries.size();
  }

private:
  struct Entry {
    Entry(const Key &key_, std::unique_ptr<Value> value_, Entry *prev_, Entry *next_): key(key_), value(std::move(value_)), prev(prev_), next(next_) {}
    Key key;
    std::unique_ptr<Value> value;
    Entry *prev;
    Entry *next;
  };

  void _removeFromQueue(Entry *entry) {
    entry->prev->next = entry->next;
    entry->next->prev = entry->prev;
  }

  //TODO Double indirection unique_ptr<Entry> and Entry has unique_ptr<Value>. Necessary?
  std::unordered_map<Key, std::unique_ptr<Entry>> _entries;
  Entry _sentinel;
};

}
}

#endif
