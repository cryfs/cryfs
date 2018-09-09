#pragma once
#ifndef MESSMER_BLOCKSTORE_TEST_TESTUTILS_GTESTPRINTERS_H_
#define MESSMER_BLOCKSTORE_TEST_TESTUTILS_GTESTPRINTERS_H_

namespace cpputils {

inline void PrintTo(const Data& /*data*/, ::std::ostream* os) {
    *os << "cpputils::Data";
}

inline void PrintTo(const boost::optional<Data>& data, ::std::ostream* os) {
    if (data == boost::none) {
        *os << "none";
    } else {
        PrintTo(*data, os);
    }
}

}

#endif
