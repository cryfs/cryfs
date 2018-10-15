#pragma once
#ifndef MESSMER_CPPUTILS_POINTER_CAST_H_
#define MESSMER_CPPUTILS_POINTER_CAST_H_

#include <memory>

namespace cpputils {

/**
 * dynamic_cast implementation for unique_ptr (moving unique_ptr into a unique_ptr of different type)
 */
//TODO Also allow passing a rvalue reference, otherwise dynamic_pointer_move(func()) won't work
template<typename DST, typename SRC>
inline std::unique_ptr<DST> dynamic_pointer_move(std::unique_ptr<SRC> &source) {
  //TODO Deleter
  DST *casted = dynamic_cast<DST*>(source.get());
  if (casted != nullptr) {
    std::ignore = source.release();
  }
  return std::unique_ptr<DST>(casted);
}

}

#endif
