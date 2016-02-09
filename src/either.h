#pragma once
#ifndef MESSMER_CPPUTILS_EITHER_H
#define MESSMER_CPPUTILS_EITHER_H

#include <boost/optional.hpp>
#include <iostream>

namespace cpputils {

    template<class Left, class Right>
    class either final {
    public:
        //TODO Try allowing construction with any type that std::is_convertible to Left or Right.
        either(const Left &left): _side(Side::left) {
            _construct_left(left);
        }
        either(Left &&left): _side(Side::left) {
            _construct_left(std::move(left));
        }
        either(const Right &right): _side(Side::right) {
            _construct_right(right);
        }
        either(Right &&right): _side(Side::right) {
            _construct_right(std::move(right));
        }
        //TODO Try allowing copy-construction when Left/Right types are std::is_convertible
        either(const either<Left, Right> &rhs): _side(rhs._side) {
            if(_side == Side::left) {
                _construct_left(rhs._left);
            } else {
                _construct_right(rhs._right);
            }
        }
        either(either<Left, Right> &&rhs): _side(rhs._side) {
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
        either<Left, Right> &operator=(const either<Left, Right> &rhs) {
            _destruct();
            _side = rhs._side;
            if (_side == Side::left) {
                _construct_left(rhs._left);
            } else {
                _construct_right(rhs._right);
            }
            return *this;
        }

        either<Left, Right> &operator=(either<Left, Right> &&rhs) {
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

        bool is_left() const {
            return _side == Side::left;
        }

        bool is_right() const {
            return _side == Side::right;
        }

        const Left &left() const& {
            return _left;
        }
        Left &left() & {
            return const_cast<Left&>(const_cast<const either<Left, Right>*>(this)->left());
        }
        Left &&left() && {
            return std::move(left());
        }

        const Right &right() const& {
            return _right;
        }
        Right &right() & {
            return const_cast<Right&>(const_cast<const either<Left, Right>*>(this)->right());
        }
        Right &&right() && {
            return std::move(right());
        }

        boost::optional<const Left&> left_opt() const& {
            if (_side == Side::left) {
                return _left;
            } else {
                return boost::none;
            }
        }
        boost::optional<Left&> left_opt() & {
            if (_side == Side::left) {
                return _left;
            } else {
                return boost::none;
            }
        }
        boost::optional<Left> left_opt() && {
            if (_side == Side::left) {
                return std::move(_left);
            } else {
                return boost::none;
            }
        }

        boost::optional<const Right&> right_opt() const& {
            if (_side == Side::right) {
                return _right;
            } else {
                return boost::none;
            }
        }
        boost::optional<Right&> right_opt() & {
            if (_side == Side::right) {
                return _right;
            } else {
                return boost::none;
            }
        }
        boost::optional<Right> right_opt() && {
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
        enum class Side : unsigned char {left, right} _side;

        either(Side side): _side(side) {}

        template<typename... Args>
        void _construct_left(Args&&... args) {
            new(&_left)Left(std::forward<Args>(args)...);
        }
        template<typename... Args>
        void _construct_right(Args&&... args) {
            new(&_right)Right(std::forward<Args>(args)...);
        }
        void _destruct() {
            if (_side == Side::left) {
                _left.~Left();
            } else {
                _right.~Right();
            }
        }

        template<typename Left_, typename Right_, typename... Args>
        friend either<Left_, Right_> make_left(Args&&... args);

        template<typename Left_, typename Right_, typename... Args>
        friend either<Left_, Right_> make_right(Args&&... args);
    };

    template<class Left, class Right>
    bool operator==(const either<Left, Right> &lhs, const either<Left, Right> &rhs) {
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
    bool operator!=(const either<Left, Right> &lhs, const either<Left, Right> &rhs) {
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
    either<Left, Right> make_left(Args&&... args) {
        either<Left, Right> result(either<Left, Right>::Side::left);
        result._construct_left(std::forward<Args>(args)...);
        return result;
    }

    template<typename Left, typename Right, typename... Args>
    either<Left, Right> make_right(Args&&... args) {
        either<Left, Right> result(either<Left, Right>::Side::right);
        result._construct_right(std::forward<Args>(args)...);
        return result;
    }
}


#endif
