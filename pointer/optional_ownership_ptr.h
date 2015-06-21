#pragma once
#ifndef MESSMER_CPP_UTILS_POINTER_OPTIONALOWNERSHIPPOINTER_H_
#define MESSMER_CPP_UTILS_POINTER_OPTIONALOWNERSHIPPOINTER_H_

#include <memory>
#include <functional>

/**
 * optional_ownership_ptr can be used to hold a pointer to an instance of an object.
 * The pointer might or might not have ownership of the object.
 *
 * If it has ownership, it will delete the stored object in its destructor.
 * If it doesn't have ownership, it won't.
 *
 * You can create such pointers with
 *   - WithOwnership(ptr)
 *   - WithoutOwnership(ptr)
 *   - null()
 */

namespace cpputils {

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

#endif
