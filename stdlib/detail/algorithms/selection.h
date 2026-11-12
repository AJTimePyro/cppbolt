#ifndef _MINI_STL_DETAIL_ALGORITHMS_SELECTION_H_
#define _MINI_STL_DETAIL_ALGORITHMS_SELECTION_H_

#include <cstddef>
#include <utility>
#include <detail/iterator.h>
#include <detail/algorithms/comp.h>
#include <detail/algorithms/heap.h>
#include <detail/algorithms/merge.h>

namespace std {

namespace detail {

template <class _RandomAccessIterator, class _Compare>
inline void __insertion_sort(_RandomAccessIterator __first, _RandomAccessIterator __last, _Compare __comp) {
    for (_RandomAccessIterator __i = __first + 1; __i < __last; ++__i) {
        auto __val = *__i;
        _RandomAccessIterator __j = __i;
        while (__j > __first && __comp(__val, *(__j - 1))) {
            *__j = *(__j - 1);
            --__j;
        }
        *__j = __val;
    }
}

} // namespace detail

template <class _RandomAccessIterator, class _Compare>
inline void partial_sort(_RandomAccessIterator __first, _RandomAccessIterator __middle,
                         _RandomAccessIterator __last, _Compare __comp) {
    std::make_heap(__first, __middle, __comp);
    ptrdiff_t __len = __middle - __first;
    for (_RandomAccessIterator __i = __middle; __i < __last; ++__i) {
        if (__comp(*__i, *__first)) {
            std::swap(*__i, *__first);
            detail::__sift_down(__first, 0, __len - 1, __comp);
        }
    }
    std::sort_heap(__first, __middle, __comp);
}

template <class _RandomAccessIterator>
inline void partial_sort(_RandomAccessIterator __first, _RandomAccessIterator __middle,
                         _RandomAccessIterator __last) {
    std::partial_sort(__first, __middle, __last, __less<>());
}

template <class _InputIterator, class _RandomAccessIterator, class _Compare>
inline _RandomAccessIterator partial_sort_copy(_InputIterator __first, _InputIterator __last,
                                               _RandomAccessIterator __result_first,
                                               _RandomAccessIterator __result_last,
                                               _Compare __comp) {
    if (__result_first == __result_last) return __result_last;
    _RandomAccessIterator __r = __result_first;
    for (; __first != __last && __r != __result_last; ++__first, ++__r) {
        *__r = *__first;
    }
    std::make_heap(__result_first, __r, __comp);
    ptrdiff_t __len = __r - __result_first;
    for (; __first != __last; ++__first) {
        if (__comp(*__first, *__result_first)) {
            *__result_first = *__first;
            detail::__sift_down(__result_first, 0, __len - 1, __comp);
        }
    }
    std::sort_heap(__result_first, __r, __comp);
    return __r;
}

template <class _InputIterator, class _RandomAccessIterator>
inline _RandomAccessIterator partial_sort_copy(_InputIterator __first, _InputIterator __last,
                                               _RandomAccessIterator __result_first,
                                               _RandomAccessIterator __result_last) {
    return std::partial_sort_copy(__first, __last, __result_first, __result_last, __less<>());
}

template <class _RandomAccessIterator, class _Compare>
inline void nth_element(_RandomAccessIterator __first, _RandomAccessIterator __nth,
                        _RandomAccessIterator __last, _Compare __comp) {
    while (__last - __first > 3) {
        _RandomAccessIterator __mid = __first + (__last - __first) / 2;
        auto __pivot = *__mid;
        _RandomAccessIterator __i = __first;
        _RandomAccessIterator __j = __last - 1;
        while (true) {
            while (__comp(*__i, __pivot)) ++__i;
            while (__comp(__pivot, *__j)) --__j;
            if (__i >= __j) break;
            std::swap(*__i, *__j);
            ++__i;
            --__j;
        }
        if (__nth < __i) {
            __last = __i;
        } else {
            __first = __i;
        }
    }
    detail::__insertion_sort(__first, __last, __comp);
}

template <class _RandomAccessIterator>
inline void nth_element(_RandomAccessIterator __first, _RandomAccessIterator __nth,
                        _RandomAccessIterator __last) {
    std::nth_element(__first, __nth, __last, __less<>());
}

template <class _RandomAccessIterator, class _Compare>
inline void stable_sort(_RandomAccessIterator __first, _RandomAccessIterator __last, _Compare __comp) {
    ptrdiff_t __len = __last - __first;
    if (__len <= 16) {
        detail::__insertion_sort(__first, __last, __comp);
        return;
    }
    _RandomAccessIterator __mid = __first + __len / 2;
    std::stable_sort(__first, __mid, __comp);
    std::stable_sort(__mid, __last, __comp);
    std::inplace_merge(__first, __mid, __last, __comp);
}

template <class _RandomAccessIterator>
inline void stable_sort(_RandomAccessIterator __first, _RandomAccessIterator __last) {
    std::stable_sort(__first, __last, __less<>());
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_SELECTION_H_
