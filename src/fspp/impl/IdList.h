#pragma once
#ifndef MESSMER_FSPP_IMPL_IDLIST_H_
#define MESSMER_FSPP_IMPL_IDLIST_H_

#include <map>
#include <mutex>
#include <stdexcept>
#include <cpp-utils/pointer/unique_ref.h>

namespace fspp {

template<class Entry>
class IdList final {
public:
  IdList();
  virtual ~IdList();

  int add(cpputils::unique_ref<Entry> entry);
  Entry *get(int id);
  const Entry *get(int id) const;
  void remove(int id);
private:
  std::map<int, cpputils::unique_ref<Entry>> _entries;
  int _id_counter;
  mutable std::mutex _mutex;

  DISALLOW_COPY_AND_ASSIGN(IdList<Entry>);
};

template<class Entry>
IdList<Entry>::IdList()
  : _entries(), _id_counter(0), _mutex() {
}

template<class Entry>
IdList<Entry>::~IdList() {
}

template<class Entry>
int IdList<Entry>::add(cpputils::unique_ref<Entry> entry) {
  std::lock_guard<std::mutex> lock(_mutex);
  //TODO Reuse IDs (ids = descriptors)
  int new_id = ++_id_counter;
  _entries.insert(std::make_pair(new_id, std::move(entry)));
  return new_id;
}

template<class Entry>
Entry *IdList<Entry>::get(int id) {
  return const_cast<Entry*>(const_cast<const IdList<Entry>*>(this)->get(id));
}

template<class Entry>
const Entry *IdList<Entry>::get(int id) const {
  std::lock_guard<std::mutex> lock(_mutex);
  const Entry *result = _entries.at(id).get();
  return result;
}

template<class Entry>
void IdList<Entry>::remove(int id) {
  std::lock_guard<std::mutex> lock(_mutex);
  auto found_iter = _entries.find(id);
  if (found_iter == _entries.end()) {
    throw std::out_of_range("Called IdList::remove() with an invalid ID");
  }
  _entries.erase(found_iter);
}

}

#endif
