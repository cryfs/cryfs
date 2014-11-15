#pragma once
#ifndef FSPP_UTILS_POINTER_H_
#define FSPP_UTILS_POINTER_H_

#include <memory>

namespace fspp {

  template<typename DST, typename SRC>
  inline std::unique_ptr<DST> dynamic_pointer_move(std::unique_ptr<SRC> &source) {
	DST *casted = dynamic_cast<DST*>(source.get());
	if (casted != nullptr) {
      source.release();
	}
	return std::unique_ptr<DST>(casted);
  }
}

#endif
