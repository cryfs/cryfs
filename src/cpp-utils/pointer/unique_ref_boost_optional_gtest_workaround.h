#pragma once
#ifndef MESSMER_CPPUTILS_POINTER_UNIQUE_REF_BOOST_OPTIONAL_GTEST_WORKAROUND_H
#define MESSMER_CPPUTILS_POINTER_UNIQUE_REF_BOOST_OPTIONAL_GTEST_WORKAROUND_H

/**
 * This is a workaround for using boost::optional<unique_ref<T>> in gtest.
 * Without including this file, the linker will fail.
 */

//TODO Test that this solves the problem (add test unit file that doesn't compile without)

#include "unique_ref.h"
//gtest/boost::optional workaround for working with optional<unique_ref<T>>
namespace cpputils {
    template<typename T>
    inline std::ostream& operator<<(std::ostream& out, const cpputils::unique_ref<T> &ref) {
      out << ref.get();
      return out;
    }
}
#include <boost/optional/optional_io.hpp>

#endif
