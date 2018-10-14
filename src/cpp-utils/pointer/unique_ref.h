#pragma once
#ifndef MESSMER_CPPUTILS_POINTER_UNIQUE_REF_H
#define MESSMER_CPPUTILS_POINTER_UNIQUE_REF_H

#include <memory>
#include <boost/optional.hpp>
#include "../macros.h"
#include "gcc_4_8_compatibility.h"
#include "cast.h"
#include "../assert/assert.h"

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
template<class T, class D = std::default_delete<T>>
class unique_ref final {
public:
    using element_type = typename std::unique_ptr<T, D>::element_type;
    using deleter_type = typename std::unique_ptr<T, D>::deleter_type;
    using pointer = typename std::unique_ptr<T, D>::pointer;

    unique_ref(unique_ref&& from) noexcept
    : _target(std::move(from._target)) {
        from._target = nullptr;
        _invariant();
    }

    template<class U> unique_ref(unique_ref<U>&& from) noexcept
    : _target(std::move(from._target)) {
        from._target = nullptr;
        _invariant();
    }

    unique_ref& operator=(unique_ref&& from) noexcept {
        _target = std::move(from._target);
        from._target = nullptr;
        _invariant();
        return *this;
    }

    template<class U> unique_ref& operator=(unique_ref<U>&& from) noexcept {
        _target = std::move(from._target);
        from._target = nullptr;
        _invariant();
        return *this;
    }

    typename std::add_lvalue_reference<element_type>::type operator*() const& noexcept {
        _invariant();
        return *_target;
    }
    typename std::add_rvalue_reference<element_type>::type operator*() && noexcept {
        _invariant();
        return std::move(*_target);
    }

    pointer operator->() const noexcept {
        return get();
    }

    pointer get() const noexcept {
        _invariant();
        return _target.get();
    }

    template<class T2>
    operator std::unique_ptr<T2>() && noexcept {
        _invariant();
        return std::move(_target);
    }

    template<class T2>
    operator std::shared_ptr<T2>() && noexcept {
        _invariant();
        return std::move(_target);
    }

    void swap(unique_ref& rhs) noexcept {
        std::swap(_target, rhs._target);
    }

    bool is_valid() const noexcept {
        return _target.get() != nullptr;
    }

    deleter_type& get_deleter() noexcept {
        return _target.get_deleter();
    }

    const deleter_type& get_deleter() const noexcept {
        return _target.get_deleter();
    }

private:
    explicit unique_ref(std::unique_ptr<T, D> target) noexcept
    : _target(std::move(target)) {}

    void _invariant() const noexcept {
        // TODO Test performance impact of this
        ASSERT(_target.get() != nullptr, "Member was moved out to another unique_ref. This instance is invalid.");
    }

    template<class U, class... Args> friend unique_ref<U> make_unique_ref(Args&&... args);
    template<class T2, class D2> friend boost::optional<unique_ref<T2, D2>> nullcheck(std::unique_ptr<T2, D2> ptr) noexcept;
    template<class T2, class D2> friend class unique_ref;
    template<class DST, class SRC> friend boost::optional<unique_ref<DST>> dynamic_pointer_move(unique_ref<SRC> &source) noexcept;
    template<class T2, class D2> friend bool operator==(const unique_ref<T2, D2>& lhs, const unique_ref<T2, D2>& rhs) noexcept;
    friend struct std::hash<unique_ref<T, D>>;
    friend struct std::less<unique_ref<T, D>>;

    std::unique_ptr<T, D> _target;

    DISALLOW_COPY_AND_ASSIGN(unique_ref);
};

template<class T, class... Args>
inline unique_ref<T> make_unique_ref(Args&&... args) {
    return unique_ref<T>(std::make_unique<T>(std::forward<Args>(args)...));
}

template<class T, class D>
inline boost::optional<unique_ref<T, D>> nullcheck(std::unique_ptr<T, D> ptr) noexcept {
    if (ptr.get() != nullptr) {
        return unique_ref<T, D>(std::move(ptr));
    }
    return boost::none;
}

template<class T, class D> inline void destruct(unique_ref<T, D> /*ptr*/) {
    // ptr will be moved in to this function and destructed on return
}

//TODO Also allow passing a rvalue reference, otherwise dynamic_pointer_move(func()) won't work
template<class DST, class SRC>
inline boost::optional<unique_ref<DST>> dynamic_pointer_move(unique_ref<SRC> &source) noexcept {
    return nullcheck<DST>(dynamic_pointer_move<DST>(source._target));
}

template<class T, class D>
inline bool operator==(const unique_ref<T, D> &lhs, const unique_ref<T, D> &rhs) noexcept {
    return lhs._target == rhs._target;
}

template<class T, class D>
inline bool operator!=(const unique_ref<T, D> &lhs, const unique_ref<T, D> &rhs) noexcept {
    return !operator==(lhs, rhs);
}

}

namespace std { // NOLINT (intentional change of namespace std)
    template<class T, class D>
    inline void swap(cpputils::unique_ref<T, D>& lhs, cpputils::unique_ref<T, D>& rhs) noexcept {
        lhs.swap(rhs);
    }

    template<class T, class D>
    inline void swap(cpputils::unique_ref<T, D>&& lhs, cpputils::unique_ref<T, D>& rhs) noexcept {
        lhs.swap(rhs);
    }

    template<class T, class D>
    inline void swap(cpputils::unique_ref<T, D>& lhs, cpputils::unique_ref<T, D>&& rhs) noexcept {
        lhs.swap(rhs);
    }

    // Allow using it in std::unordered_set / std::unordered_map
    template<class T, class D> struct hash<cpputils::unique_ref<T, D>> {
        size_t operator()(const cpputils::unique_ref<T, D> &ref) const noexcept {
            return std::hash<unique_ptr<T, D>>()(ref._target);
        }
    };

    // Allow using it in std::map / std::set
    template <class T, class D> struct less<cpputils::unique_ref<T, D>> {
        bool operator()(const cpputils::unique_ref<T, D> &lhs, const cpputils::unique_ref<T, D> &rhs) const noexcept {
            return lhs._target < rhs._target;
        }
    };
}

#endif
