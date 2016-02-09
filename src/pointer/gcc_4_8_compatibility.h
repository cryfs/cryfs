#pragma once
#ifndef MESSMER_CPPUTILS_GCC48COMPATIBILITY_H
#define MESSMER_CPPUTILS_GCC48COMPATIBILITY_H

#include <memory>

#if __GNUC__ == 4 && __GNUC_MINOR__ == 8
// Add std::make_unique
namespace std {
    template<typename T, typename... Args>
    inline unique_ptr<T> make_unique(Args&&... args) {
        return unique_ptr<T>(new T(std::forward<Args>(args)...));
    }
}

#endif

#endif
