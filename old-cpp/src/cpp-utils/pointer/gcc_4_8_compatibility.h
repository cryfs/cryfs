#pragma once
#ifndef MESSMER_CPPUTILS_GCC48COMPATIBILITY_H
#define MESSMER_CPPUTILS_GCC48COMPATIBILITY_H

#include <memory>

#if __GNUC__ == 4 && __GNUC_MINOR__ == 8
// Add std::make_unique
namespace std {
  /// Alias template for remove_extent
  template<typename _Tp>
    using remove_extent_t = typename remove_extent<_Tp>::type;

  template<typename _Tp>
    struct _MakeUniq
    { typedef unique_ptr<_Tp> __single_object; };

  template<typename _Tp>
    struct _MakeUniq<_Tp[]>
    { typedef unique_ptr<_Tp[]> __array; };

  template<typename _Tp, size_t _Bound>
    struct _MakeUniq<_Tp[_Bound]>
    { struct __invalid_type { }; };

  /// std::make_unique for single objects
  template<typename _Tp, typename... _Args>
    inline typename _MakeUniq<_Tp>::__single_object
    make_unique(_Args&&... __args)
    { return unique_ptr<_Tp>(new _Tp(std::forward<_Args>(__args)...)); }

  /// std::make_unique for arrays of unknown bound
  template<typename _Tp>
    inline typename _MakeUniq<_Tp>::__array
    make_unique(size_t __num)
    { return unique_ptr<_Tp>(new remove_extent_t<_Tp>[__num]()); }

  /// Disable std::make_unique for arrays of known bound
  template<typename _Tp, typename... _Args>
    inline typename _MakeUniq<_Tp>::__invalid_type
    make_unique(_Args&&...) = delete;
}

#endif

#endif
