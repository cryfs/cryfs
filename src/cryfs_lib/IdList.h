#ifndef CRYFS_LIB_IDLIST_H_
#define CRYFS_LIB_IDLIST_H_

#include <map>
#include <memory>
#include "utils/macros.h"

namespace cryfs {

template<class Entry>
class IdList {
public:
  IdList();
  virtual ~IdList();

  int add(std::unique_ptr<Entry> entry);
  Entry *get(int id);
  const Entry *get(int id) const;
  void remove(int id);
private:
  std::map<int, std::unique_ptr<Entry>> _entries;

  DISALLOW_COPY_AND_ASSIGN(IdList<Entry>)
};

template<class Entry>
IdList<Entry>::IdList()
  : _entries() {
}

template<class Entry>
IdList<Entry>::~IdList() {
}

template<class Entry>
int IdList<Entry>::add(std::unique_ptr<Entry> entry) {
  //TODO Reuse IDs (ids = descriptors)
  int new_id = _entries.size();
  _entries[new_id] = std::move(entry);
  return new_id;
}

template<class Entry>
Entry *IdList<Entry>::get(int id) {
  return const_cast<Entry*>(const_cast<const IdList<Entry>*>(this)->get(id));
}

template<class Entry>
const Entry *IdList<Entry>::get(int id) const {
  return _entries.at(id).get();
}

template<class Entry>
void IdList<Entry>::remove(int id) {
  _entries.erase(id);
}

} /* namespace cryfs */

#endif /* CRYFS_LIB_IDLIST_H_ */
