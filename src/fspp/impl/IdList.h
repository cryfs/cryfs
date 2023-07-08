#pragma once
#ifndef MESSMER_FSPP_IMPL_IDLIST_H_
#define MESSMER_FSPP_IMPL_IDLIST_H_

#include <unordered_map>
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
  size_t size() const;

private:
  std::unordered_map<int, cpputils::unique_ref<Entry>> _entries;
  int _id_counter;

  DISALLOW_COPY_AND_ASSIGN(IdList<Entry>);
};

template<class Entry>
IdList<Entry>::IdList()
  : _entries(), _id_counter(0) {
}

template<class Entry>
IdList<Entry>::~IdList() {
}

template<class Entry>
int IdList<Entry>::add(cpputils::unique_ref<Entry> entry) {
  //TODO Reuse IDs (ids = descriptors)
  const int new_id = ++_id_counter;
  _entries.emplace(new_id, std::move(entry));
  return new_id;
}

template<class Entry>
Entry *IdList<Entry>::get(int id) {
  return const_cast<Entry*>(const_cast<const IdList<Entry>*>(this)->get(id));
}

template<class Entry>
const Entry *IdList<Entry>::get(int id) const {
  const Entry *result = _entries.at(id).get();
  return result;
}

template<class Entry>
void IdList<Entry>::remove(int id) {
  auto found_iter = _entries.find(id);
  if (found_iter == _entries.end()) {
    throw std::out_of_range("Called IdList::remove() with an invalid ID");
  }
  _entries.erase(found_iter);
}

template<class Entry>
size_t IdList<Entry>::size() const {
	return _entries.size();
}

}

#endif
