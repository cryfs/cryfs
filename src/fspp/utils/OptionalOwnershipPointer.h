#pragma once
#ifndef FSPP_UTILS_OPTIONALOWNERSHIPPOINTER_H_
#define FSPP_UTILS_OPTIONALOWNERSHIPPOINTER_H_

#include <memory>

namespace fspp {
namespace ptr {

template<typename T>
using optional_ownership_ptr = std::unique_ptr<T, std::function<void(T*)>>;

template<typename T>
optional_ownership_ptr<T> WithOwnership(std::unique_ptr<T> obj) {
  auto deleter = obj.get_deleter();
  return optional_ownership_ptr<T>(obj.release(), [deleter](T* obj){deleter(obj);});
}

template<typename T>
optional_ownership_ptr<T> WithoutOwnership(T *obj) {
  return optional_ownership_ptr<T>(obj, [](T*){});
}

template<typename T>
optional_ownership_ptr<T> null() {
  return WithoutOwnership<T>(nullptr);
}

}
}

#endif
