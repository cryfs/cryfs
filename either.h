#pragma once
#ifndef MESSMER_CPP_UTILS_EITHER_H
#define MESSMER_CPP_UTILS_EITHER_H

namespace cpputils {

    template<class Left, class Right>
    class Either final {
    public:
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
        //TODO Test copy assignment operator
        //TODO Copy assignment operator
        //TODO Test destruction after copy assignment

        //TODO Test move assignment operator
        //TODO Move assignment operator
        //TODO Test destruction after move assignment

        //TODO Test operator==/operator!=
        //TODO operator==/operator!=

        //TODO Test operator<<
        //TODO operator<<(ostream)

        ~Either() {
            if (_side == Side::left) {
                _left.~Left();
            } else {
                _right.~Right();
            }
        }

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

        //TODO Test const and non-const left()/right()
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

    //TODO Test make_either<>
    //TODO make_either<>
}


#endif
