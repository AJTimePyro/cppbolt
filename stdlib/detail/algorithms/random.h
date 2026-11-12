#ifndef _MINI_STL_DETAIL_ALGORITHMS_RANDOM_H_
#define _MINI_STL_DETAIL_ALGORITHMS_RANDOM_H_

#include <cstddef>
#include <utility>

namespace std {

namespace detail {

template <class _URBG>
inline unsigned long long __uniform_below(_URBG& __g, unsigned long long __n) {
    if (__n <= 1) return 0;
    unsigned long long __min = (unsigned long long)__g.min();
    unsigned long long __range = (unsigned long long)__g.max() - __min + 1ULL;
    if (__range == 0) return (unsigned long long)__g() % __n;
    unsigned long long __limit = __range - (__range % __n);
    unsigned long long __value = (unsigned long long)__g() - __min;
    while (__value >= __limit) {
        __value = (unsigned long long)__g() - __min;
    }
    return __value % __n;
}

} // namespace detail

template <class _RandomAccessIterator, class _UniformRandomBitGenerator>
inline void shuffle(_RandomAccessIterator __first, _RandomAccessIterator __last,
                    _UniformRandomBitGenerator&& __g) {
    ptrdiff_t __n = __last - __first;
    for (ptrdiff_t __i = __n - 1; __i > 0; --__i) {
        ptrdiff_t __j = (ptrdiff_t)detail::__uniform_below(__g, (unsigned long long)(__i + 1));
        std::swap(*(__first + __i), *(__first + __j));
    }
}

template <class _InputIterator, class _RandomAccessIterator, class _Distance,
          class _UniformRandomBitGenerator>
inline _RandomAccessIterator sample(_InputIterator __first, _InputIterator __last,
                                    _RandomAccessIterator __out, _Distance __n,
                                    _UniformRandomBitGenerator&& __g) {
    _Distance __k = 0;
    for (; __first != __last && __k < __n; ++__first, ++__k) {
        __out[__k] = *__first;
    }
    _Distance __sz = __k;
    for (; __first != __last; ++__first, ++__sz) {
        _Distance __r = (_Distance)detail::__uniform_below(__g, (unsigned long long)(__sz + 1));
        if (__r < __n) {
            __out[__r] = *__first;
        }
    }
    return __out + __k;
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_RANDOM_H_
