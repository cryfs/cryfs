#pragma once
#ifndef MESSMER_CPP_UTILS_EITHER_H
#define MESSMER_CPP_UTILS_EITHER_H

#include <type_traits>

namespace cpputils {

    template<class Left, class Right>
    class Either final {
    public:
        //TODO Try allowing construction with any type that std::is_convertible to Left or Right.
        Either(const Left &left): _side(Side::left) {
            new(&_left)Left(left);
        }
        Either(Left &&left): _side(Side::left) {
            new(&_left)Left(std::move(left));
        }
        Either(const Right &right): _side(Side::right) {
            new(&_right)Right(right);
        }
        Either(Right &&right): _side(Side::right) {
            new(&_right)Right(std::move(right));
        }
        //TODO Try allowing copy-construction when Left/Right types are std::is_convertible
        Either(const Either<Left, Right> &rhs): _side(rhs._side) {
            if(_side == Side::left) {
                new(&_left)Left(rhs._left);
            } else {
                new(&_right)Right(rhs._right);
            }
        }
        Either(Either<Left, Right> &&rhs): _side(rhs._side) {
            if(_side == Side::left) {
                new(&_left)Left(std::move(rhs._left));
            } else {
                new(&_right)Right(std::move(rhs._right));
            }
        }

        ~Either() {
            if (_side == Side::left) {
                _left.~Left();
            } else {
                _right.~Right();
            }
        }

        //TODO Test copy assignment operator
        //TODO Copy assignment operator
        //TODO Test destruction after copy assignment

        //TODO Test move assignment operator
        //TODO Move assignment operator
        //TODO Test destruction after move assignment

        //TODO Test operator<<
        //TODO operator<<(ostream)

        bool is_left() const {
            return _side == Side::left;
        }

        bool is_right() const {
            return _side == Side::right;
        }

        //TODO Also offer a safe version of getting left/right (exceptions? nullptr?)
        const Left &left() const {
            return _left;
        }

        const Right &right() const {
            return _right;
        }

        Left &left() {
            return const_cast<Left&>(const_cast<const Either<Left, Right>*>(this)->left());
        }
        Right &right() {
            return const_cast<Right&>(const_cast<const Either<Left, Right>*>(this)->right());
        }
    private:
        union {
            Left _left;
            Right _right;
        };
        enum class Side : unsigned char {left, right} _side;
    };

    template<class Left, class Right>
    bool operator==(const Either<Left, Right> &lhs, const Either<Left, Right> &rhs) {
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
    bool operator!=(const Either<Left, Right> &lhs, const Either<Left, Right> &rhs) {
        return !operator==(lhs, rhs);
    }

    //TODO Test make_either<>
    //TODO make_either<>
}


#endif
