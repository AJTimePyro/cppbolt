#ifndef _MINI_STL_DETAIL_ALGORITHMS_RANDOM_H_
#define _MINI_STL_DETAIL_ALGORITHMS_RANDOM_H_

#include <cstddef>
#include <utility>

namespace std {

template <class _RandomAccessIterator, class _UniformRandomBitGenerator>
inline void shuffle(_RandomAccessIterator __first, _RandomAccessIterator __last,
                    _UniformRandomBitGenerator&& __g) {
    ptrdiff_t __n = __last - __first;
    for (ptrdiff_t __i = __n - 1; __i > 0; --__i) {
        ptrdiff_t __j = (ptrdiff_t)(__g() % (unsigned long long)(__i + 1));
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
        _Distance __r = (_Distance)(__g() % (unsigned long long)(__sz + 1));
        if (__r < __n) {
            __out[__r] = *__first;
        }
    }
    return __out + __k;
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_RANDOM_H_
