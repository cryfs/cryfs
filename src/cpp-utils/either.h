#pragma once
#ifndef MESSMER_CPPUTILS_EITHER_H
#define MESSMER_CPPUTILS_EITHER_H

#include <boost/optional.hpp>
#include <iostream>
#include "assert/assert.h"

namespace cpputils {

    template<class Left, class Right>
    class either final {
    public:
        //TODO Try allowing construction with any type that std::is_convertible to Left or Right.
        either(const Left &left) noexcept(noexcept(std::declval<either<Left, Right>>()._construct_left(left)))
                : _side(Side::left) {
            _construct_left(left);
        }

        either(Left &&left) noexcept(noexcept(std::declval<either<Left, Right>>()._construct_left(std::move(left))))
                : _side(Side::left) {
            _construct_left(std::move(left));
        }

        either(const Right &right) noexcept(noexcept(std::declval<either<Left, Right>>()._construct_right(right)))
                : _side(Side::right) {
            _construct_right(right);
        }

        either(Right &&right) noexcept(noexcept(std::declval<either<Left, Right>>()._construct_right(std::move(right))))
                : _side(Side::right) {
            _construct_right(std::move(right));
        }

        //TODO Try allowing copy-construction when Left/Right types are std::is_convertible
        either(const either<Left, Right> &rhs) noexcept(noexcept(std::declval<either<Left, Right>>()._construct_left(rhs._left)) && noexcept(std::declval<either<Left, Right>>()._construct_right(rhs._right)))
                : _side(rhs._side) {
            if(_side == Side::left) {
                _construct_left(rhs._left);
            } else {
                _construct_right(rhs._right);
            }
        }

        either(either<Left, Right> &&rhs) noexcept(noexcept(_construct_left(std::move(rhs._left))) && noexcept(_construct_right(std::move(rhs._right))))
                : _side(rhs._side) {
            if(_side == Side::left) {
                _construct_left(std::move(rhs._left));
            } else {
                _construct_right(std::move(rhs._right));
            }
        }

        ~either() {
            _destruct();
        }

        //TODO Try allowing copy-assignment when Left/Right types are std::is_convertible
        either<Left, Right> &operator=(const either<Left, Right> &rhs) noexcept(noexcept(_construct_left(rhs._left)) && noexcept(_construct_right(rhs._right))) {
            _destruct();
            _side = rhs._side;
            if (_side == Side::left) {
                _construct_left(rhs._left);
            } else {
                _construct_right(rhs._right);
            }
            return *this;
        }

        either<Left, Right> &operator=(either<Left, Right> &&rhs) noexcept(noexcept(_construct_left(std::move(rhs._left))) && noexcept(_construct_right(std::move(rhs._right)))) {
            _destruct();
            _side = rhs._side;
            if (_side == Side::left) {
                _construct_left(std::move(rhs._left));
            } else {
                _construct_right(std::move(rhs._right));
            }
            return *this;
        }

        //TODO fold, map_left, map_right, left_or_else(val), right_or_else(val), left_or_else(func), right_or_else(func)

        bool is_left() const noexcept {
            return _side == Side::left;
        }

        bool is_right() const noexcept {
            return _side == Side::right;
        }

        const Left &left() const& {
            ASSERT(is_left(), "Tried to get left side of an either which is right.");
            return _left;
        }
        Left &left() & {
            return const_cast<Left&>(const_cast<const either<Left, Right>*>(this)->left());
        }
        Left &&left() && {
            return std::move(left());
        }

        const Right &right() const& {
            ASSERT(is_right(), "Tried to get right side of an either which is left.");
            return _right;
        }
        Right &right() & {
            return const_cast<Right&>(const_cast<const either<Left, Right>*>(this)->right());
        }
        Right &&right() && {
            return std::move(right());
        }

        boost::optional<const Left&> left_opt() const& noexcept {
            if (_side == Side::left) {
                return _left;
            } else {
                return boost::none;
            }
        }
        boost::optional<Left&> left_opt() & noexcept {
            if (_side == Side::left) {
                return _left;
            } else {
                return boost::none;
            }
        }
        boost::optional<Left> left_opt() && noexcept {
            if (_side == Side::left) {
                return std::move(_left);
            } else {
                return boost::none;
            }
        }

        boost::optional<const Right&> right_opt() const& noexcept {
            if (_side == Side::right) {
                return _right;
            } else {
                return boost::none;
            }
        }
        boost::optional<Right&> right_opt() & noexcept {
            if (_side == Side::right) {
                return _right;
            } else {
                return boost::none;
            }
        }
        boost::optional<Right> right_opt() && noexcept {
            if (_side == Side::right) {
                return std::move(_right);
            } else {
                return boost::none;
            }
        }

    private:
        union {
            Left _left;
            Right _right;
        };
        enum class Side : uint8_t {left, right} _side;

        explicit either(Side side) noexcept : _side(side) {}

        template<typename... Args>
        void _construct_left(Args&&... args) noexcept(noexcept(new Left(std::forward<Args>(args)...))) {
            new(&_left)Left(std::forward<Args>(args)...);
        }
        template<typename... Args>
        void _construct_right(Args&&... args) noexcept(noexcept(new Right(std::forward<Args>(args)...))) {
            new(&_right)Right(std::forward<Args>(args)...);
        }
        void _destruct() noexcept {
            if (_side == Side::left) {
                _left.~Left();
            } else {
                _right.~Right();
            }
        }

        template<class... Args> static constexpr bool left_noexcept_constructible(Args&&... args) {
            return noexcept(_construct_left(std::forward<Args>(args)...));
        }

        template<typename Left_, typename Right_, typename... Args>
        friend either<Left_, Right_> make_left(Args&&... args) noexcept(noexcept(std::declval<either<Left, Right>>()._construct_left(std::forward<Args>(args)...)));

        template<typename Left_, typename Right_, typename... Args>
        friend either<Left_, Right_> make_right(Args&&... args) noexcept(noexcept(std::declval<either<Left, Right>>()._construct_right(std::forward<Args>(args)...)));
    };

    template<class Left, class Right>
    bool operator==(const either<Left, Right> &lhs, const either<Left, Right> &rhs) noexcept(noexcept(std::declval<Left>() == std::declval<Left>()) && noexcept(std::declval<Right>() == std::declval<Right>())) {
        if (lhs.is_left() != rhs.is_left()) {
            return false;
        }
        if (lhs.is_left()) {
            return lhs.left() == rhs.left();
        } else {
            return lhs.right() == rhs.right();
        }
    }

    template<class Left, class Right>
    bool operator!=(const either<Left, Right> &lhs, const either<Left, Right> &rhs) noexcept(noexcept(operator==(lhs, rhs))) {
        return !operator==(lhs, rhs);
    }

    template<class Left, class Right>
    std::ostream &operator<<(std::ostream &stream, const either<Left, Right> &value) {
        if (value.is_left()) {
            stream << "Left(" << value.left() << ")";
        } else {
            stream << "Right(" << value.right() << ")";
        }
        return stream;
    }

    template<typename Left, typename Right, typename... Args>
    either<Left, Right> make_left(Args&&... args) noexcept(noexcept(std::declval<either<Left, Right>>()._construct_left(std::forward<Args>(args)...))) {
        either<Left, Right> result(either<Left, Right>::Side::left);
        result._construct_left(std::forward<Args>(args)...);
        return result;
    }

    template<typename Left, typename Right, typename... Args>
    either<Left, Right> make_right(Args&&... args) noexcept(noexcept(std::declval<either<Left, Right>>()._construct_right(std::forward<Args>(args)...))) {
        either<Left, Right> result(either<Left, Right>::Side::right);
        result._construct_right(std::forward<Args>(args)...);
        return result;
    }
}


#endif
