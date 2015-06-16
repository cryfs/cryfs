#ifndef MESSMER_CPP_UTILS_UNIQUE_REF_H
#define MESSMER_CPP_UTILS_UNIQUE_REF_H

#include <memory>
#include <boost/optional.hpp>
#include "macros.h"

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
class unique_ref {
public:

    unique_ref(unique_ref&& from): _target(std::move(from._target)) {}

    unique_ref& operator=(unique_ref&& from) {
        _target = from._target;
    }

    typename std::add_lvalue_reference<T>::type operator*() const {
        return *_target;
    }

    T* operator->() const {
        return get();
    }

    T* get() const {
        return _target.get();
    }

    T* release() {
        return _target.release();
    }

    void swap(unique_ref&& rhs) {
        _target.swap(rhs._target);
    }

private:
    unique_ref(std::unique_ptr<T> target): _target(std::move(target)) {}
    template<typename U, typename... Args> friend unique_ref<U> make_unique_ref(Args&&... args);
    template<typename U> friend boost::optional<unique_ref<U>> nullcheck(std::unique_ptr<U> ptr);

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

template<typename T1, typename T2>
inline bool operator==(const unique_ref<T1>& lhs, const unique_ref<T2>& rhs) {
   return lhs.get() == rhs.get();
}

template<typename T1, typename T2>
inline bool operator!=(const unique_ref<T1>& lhs, const unique_ref<T2>& rhs) {
   return !operator==(lhs, rhs);
}

template<typename T1, typename T2>
inline bool operator<(const unique_ref<T1>& lhs, const unique_ref<T2>& rhs) {
   return lhs.get() < rhs.get();
}

template<typename T1, typename T2>
inline bool operator<=(const unique_ref<T1>& lhs, const unique_ref<T2>& rhs) {
   return !operator<(rhs, lhs);
}

template<typename T1, typename T2>
inline bool operator>(const unique_ref<T1>& lhs, const unique_ref<T2>& rhs) {
   return operator<(rhs, lhs);
}

template<typename T1, typename T2>
inline bool operator>=(const unique_ref<T1>& lhs, const unique_ref<T2>& rhs) {
   return !operator<(lhs, rhs);
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
}

#endif
