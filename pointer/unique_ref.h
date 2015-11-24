#pragma once
#ifndef MESSMER_CPPUTILS_POINTER_UNIQUE_REF_H
#define MESSMER_CPPUTILS_POINTER_UNIQUE_REF_H

#include <memory>
#include <boost/optional.hpp>
#include "../macros.h"
#include "gcc_4_8_compatibility.h"
#include "cast.h"

namespace cpputils {

/**
 * unique_ref<T> behaves like unique_ptr<T>, but guarantees that the pointer points to a valid object.
 * You can create objects using make_unique_ref (works like make_unique for unique_ptr).
 *
 * If you happen to already have a unique_ptr<T>, you can call nullcheck(unique_ptr),
 * which returns optional<unique_ref<T>>.
 * Take care that this should be used very rarely, since it circumvents parts of the guarantee.
 * It still protects against null pointers, but it does not guarantee anymore that the pointer points
 * to a valid object. It might hold an arbitrary non-null memory location.
 *
 * Caution: There is one way a unique_ref<T> can actually hold a nullptr.
 * It will hold a nullptr after its value was moved to another unique_ref.
 * Never use the old instance after moving!
 */
template<typename T>
class unique_ref final {
public:

    unique_ref(unique_ref&& from): _target(std::move(from._target)) {}
    // TODO Test this upcast-allowing move constructor
    template<typename U> unique_ref(unique_ref<U>&& from): _target(std::move(from._target)) {}

    unique_ref& operator=(unique_ref&& from) {
        _target = std::move(from._target);
        return *this;
    }
    // TODO Test this upcast-allowing assignment
    template<typename U> unique_ref& operator=(unique_ref<U>&& from) {
        _target = std::move(from._target);
        return *this;
    }

    typename std::add_lvalue_reference<T>::type operator*() const& {
        return *_target;
    }
    typename std::add_rvalue_reference<T>::type operator*() && {
        return std::move(*_target);
    }

    T* operator->() const {
        return get();
    }

    T* get() const {
        return _target.get();
    }

    void swap(unique_ref& rhs) {
        std::swap(_target, rhs._target);
    }

private:
    unique_ref(std::unique_ptr<T> target): _target(std::move(target)) {}
    template<typename U, typename... Args> friend unique_ref<U> make_unique_ref(Args&&... args);
    template<typename U> friend boost::optional<unique_ref<U>> nullcheck(std::unique_ptr<U> ptr);
    template<typename U> friend class unique_ref;
    template<typename DST, typename SRC> friend boost::optional<unique_ref<DST>> dynamic_pointer_move(unique_ref<SRC> &source);
    template<typename U> friend std::unique_ptr<U> to_unique_ptr(unique_ref<U> ref);

    std::unique_ptr<T> _target;

    DISALLOW_COPY_AND_ASSIGN(unique_ref);
};

template<typename T, typename... Args>
inline unique_ref<T> make_unique_ref(Args&&... args) {
    return unique_ref<T>(std::make_unique<T>(std::forward<Args>(args)...));
}

template<typename T>
inline boost::optional<unique_ref<T>> nullcheck(std::unique_ptr<T> ptr) {
    if (ptr.get() != nullptr) {
        return unique_ref<T>(std::move(ptr));
    }
    return boost::none;
}

template<typename T> inline void destruct(unique_ref<T> ptr) {
   to_unique_ptr(std::move(ptr)).reset();
}

//TODO Also allow passing a rvalue reference, otherwise dynamic_pointer_move(func()) won't work
template<typename DST, typename SRC>
inline boost::optional<unique_ref<DST>> dynamic_pointer_move(unique_ref<SRC> &source) {
    return nullcheck<DST>(dynamic_pointer_move<DST>(source._target));
}

//TODO Write test cases for to_unique_ptr
template<typename T>
inline std::unique_ptr<T> to_unique_ptr(unique_ref<T> ref) {
    return std::move(ref._target);
}

template<typename T>
inline bool operator==(const unique_ref<T> &lhs, const unique_ref<T> &rhs) {
    return lhs.get() == rhs.get();
}

template<typename T>
inline bool operator!=(const unique_ref<T> &lhs, const unique_ref<T> &rhs) {
    return !operator==(lhs, rhs);
}

}

namespace std {
    template<typename T>
    inline void swap(cpputils::unique_ref<T>& lhs, cpputils::unique_ref<T>& rhs) {
        lhs.swap(rhs);
    }

    template<typename T>
    inline void swap(cpputils::unique_ref<T>&& lhs, cpputils::unique_ref<T>& rhs) {
        lhs.swap(rhs);
    }

    template<typename T>
    inline void swap(cpputils::unique_ref<T>& lhs, cpputils::unique_ref<T>&& rhs) {
        lhs.swap(rhs);
    }

    // Allow using it in std::unordered_set / std::unordered_map
    template<typename T> struct hash<cpputils::unique_ref<T>> {
        size_t operator()(const cpputils::unique_ref<T> &ref) const {
            return (size_t)ref.get();
        }
    };

    // Allow using it in std::map / std::set
    template <typename T> struct less<cpputils::unique_ref<T>> {
        bool operator()(const cpputils::unique_ref<T> &lhs, const cpputils::unique_ref<T> &rhs) const {
            return lhs.get() < rhs.get();
        }
    };
}

#endif
