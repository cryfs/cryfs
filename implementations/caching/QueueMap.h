#pragma once
#ifndef MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING2_MAP_H_
#define MESSMER_BLOCKSTORE_IMPLEMENTATIONS_CACHING2_MAP_H_

#include <memory>
#include <unordered_map>
#include <cassert>
#include <boost/optional.hpp>
#include <messmer/cpp-utils/macros.h>

namespace blockstore {
namespace caching {

//TODO Test
//TODO Move to utils
template<class Key, class Value>
class QueueMap {
public:
  QueueMap(): _entries(), _sentinel(&_sentinel, &_sentinel) {
  }
  virtual ~QueueMap() {
    for (auto &entry : _entries) {
      entry.second.release();
    }
  }

  void push(const Key &key, Value value) {
    auto newEntry = _entries.emplace(std::piecewise_construct, std::forward_as_tuple(key), std::forward_as_tuple(_sentinel.prev, &_sentinel));
    assert(newEntry.second == true);
    newEntry.first->second.init(&newEntry.first->first, std::move(value));
    //The following is ok, because std::unordered_map never invalidates pointers to its entries
    _sentinel.prev->next = &newEntry.first->second;
    _sentinel.prev = &newEntry.first->second;
  }

  boost::optional<Value> pop(const Key &key) {
    auto found = _entries.find(key);
    if (found == _entries.end()) {
      return boost::none;
    }
    _removeFromQueue(found->second);
    auto value = found->second.release();
    _entries.erase(found);
    return std::move(value);
  }

  boost::optional<Value> pop() {
    if(_sentinel.next == &_sentinel) {
      return boost::none;
    }
    return pop(*_sentinel.next->key);
  }

  boost::optional<const Value &> peek() {
    if(_sentinel.next == &_sentinel) {
      return boost::none;
    }
    return _sentinel.next->value();
  }

  uint32_t size() {
    return _entries.size();
  }

private:
  class Entry {
  public:
    Entry(Entry *prev_, Entry *next_): prev(prev_), next(next_), key(nullptr), _value() {
    }
    void init(const Key *key_, Value value_) {
      key = key_;
      new(_value) Value(std::move(value_));
    }
    Value release() {
      Value value = std::move(*reinterpret_cast<Value*>(_value));
      reinterpret_cast<Value*>(_value)->~Value();
      return value;
    }
    const Value &value() {
      return *reinterpret_cast<Value*>(_value);
    }
    Entry *prev;
    Entry *next;
    const Key *key;
  private:
    alignas(Value) char _value[sizeof(Value)];
    DISALLOW_COPY_AND_ASSIGN(Entry);
  };

  void _removeFromQueue(const Entry &entry) {
    entry.prev->next = entry.next;
    entry.next->prev = entry.prev;
  }

  std::unordered_map<Key, Entry> _entries;
  Entry _sentinel;

  DISALLOW_COPY_AND_ASSIGN(QueueMap);
};

}
}

#endif
