#ifndef _MINI_STL_DETAIL_ALGORITHMS_BINARY_SEARCH_H_
#define _MINI_STL_DETAIL_ALGORITHMS_BINARY_SEARCH_H_

#include <cstddef>
#include <utility>
#include <detail/iterator.h>
#include <detail/algorithms/comp.h>

namespace std {

template <class _ForwardIterator, class _Tp, class _Compare>
inline _ForwardIterator lower_bound(_ForwardIterator __first, _ForwardIterator __last,
                                    const _Tp& __value, _Compare __comp) {
    ptrdiff_t __len = std::distance(__first, __last);
    while (__len > 0) {
        ptrdiff_t __half = __len / 2;
        _ForwardIterator __middle = std::next(__first, __half);
        if (__comp(*__middle, __value)) {
            __first = std::next(__middle);
            __len = __len - __half - 1;
        } else {
            __len = __half;
        }
    }
    return __first;
}

template <class _ForwardIterator, class _Tp>
inline _ForwardIterator lower_bound(_ForwardIterator __first, _ForwardIterator __last, const _Tp& __value) {
    return std::lower_bound(__first, __last, __value, __less<>());
}

template <class _ForwardIterator, class _Tp, class _Compare>
inline _ForwardIterator upper_bound(_ForwardIterator __first, _ForwardIterator __last,
                                    const _Tp& __value, _Compare __comp) {
    ptrdiff_t __len = std::distance(__first, __last);
    while (__len > 0) {
        ptrdiff_t __half = __len / 2;
        _ForwardIterator __middle = std::next(__first, __half);
        if (__comp(__value, *__middle)) {
            __len = __half;
        } else {
            __first = std::next(__middle);
            __len = __len - __half - 1;
        }
    }
    return __first;
}

template <class _ForwardIterator, class _Tp>
inline _ForwardIterator upper_bound(_ForwardIterator __first, _ForwardIterator __last, const _Tp& __value) {
    return std::upper_bound(__first, __last, __value, __less<>());
}

template <class _ForwardIterator, class _Tp, class _Compare>
inline bool binary_search(_ForwardIterator __first, _ForwardIterator __last,
                          const _Tp& __value, _Compare __comp) {
    __first = std::lower_bound(__first, __last, __value, __comp);
    return __first != __last && !__comp(__value, *__first);
}

template <class _ForwardIterator, class _Tp>
inline bool binary_search(_ForwardIterator __first, _ForwardIterator __last, const _Tp& __value) {
    return std::binary_search(__first, __last, __value, __less<>());
}

template <class _ForwardIterator, class _Tp, class _Compare>
inline pair<_ForwardIterator, _ForwardIterator>
equal_range(_ForwardIterator __first, _ForwardIterator __last,
            const _Tp& __value, _Compare __comp) {
    return pair<_ForwardIterator, _ForwardIterator>(
        std::lower_bound(__first, __last, __value, __comp),
        std::upper_bound(__first, __last, __value, __comp));
}

template <class _ForwardIterator, class _Tp>
inline pair<_ForwardIterator, _ForwardIterator>
equal_range(_ForwardIterator __first, _ForwardIterator __last, const _Tp& __value) {
    return std::equal_range(__first, __last, __value, __less<>());
}

template <class _ForwardIterator, class _Compare>
inline _ForwardIterator is_sorted_until(_ForwardIterator __first, _ForwardIterator __last, _Compare __comp) {
    if (__first != __last) {
        _ForwardIterator __next = __first;
        for (++__next; __next != __last; __first = __next, ++__next) {
            if (__comp(*__next, *__first)) {
                return __next;
            }
        }
    }
    return __last;
}

template <class _ForwardIterator>
inline _ForwardIterator is_sorted_until(_ForwardIterator __first, _ForwardIterator __last) {
    return std::is_sorted_until(__first, __last, __less<>());
}

template <class _ForwardIterator, class _Compare>
inline bool is_sorted(_ForwardIterator __first, _ForwardIterator __last, _Compare __comp) {
    return std::is_sorted_until(__first, __last, __comp) == __last;
}

template <class _ForwardIterator>
inline bool is_sorted(_ForwardIterator __first, _ForwardIterator __last) {
    return std::is_sorted(__first, __last, __less<>());
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_BINARY_SEARCH_H_
