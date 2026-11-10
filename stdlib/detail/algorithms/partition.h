#ifndef _MINI_STL_DETAIL_ALGORITHMS_PARTITION_H_
#define _MINI_STL_DETAIL_ALGORITHMS_PARTITION_H_

#include <cstddef>
#include <utility>
#include <detail/iterator.h>
#include <detail/algorithms/modifying.h>

namespace std {

template <class _InputIterator, class _Predicate>
inline bool is_partitioned(_InputIterator __first, _InputIterator __last, _Predicate __pred) {
    for (; __first != __last; ++__first) {
        if (!__pred(*__first)) break;
    }
    for (; __first != __last; ++__first) {
        if (__pred(*__first)) return false;
    }
    return true;
}

template <class _ForwardIterator, class _Predicate>
inline _ForwardIterator partition(_ForwardIterator __first, _ForwardIterator __last, _Predicate __pred) {
    while (true) {
        while (__first != __last && __pred(*__first)) {
            ++__first;
        }
        if (__first == __last) break;
        _ForwardIterator __it = __first;
        for (++__it; __it != __last; ++__it) {
            if (__pred(*__it)) {
                std::iter_swap(__first, __it);
                ++__first;
                break;
            }
        }
        if (__it == __last) break;
    }
    return __first;
}

template <class _InputIterator, class _OutputIterator1, class _OutputIterator2, class _Predicate>
inline pair<_OutputIterator1, _OutputIterator2>
partition_copy(_InputIterator __first, _InputIterator __last,
               _OutputIterator1 __out_true, _OutputIterator2 __out_false,
               _Predicate __pred) {
    for (; __first != __last; ++__first) {
        if (__pred(*__first)) {
            *__out_true = *__first;
            ++__out_true;
        } else {
            *__out_false = *__first;
            ++__out_false;
        }
    }
    return pair<_OutputIterator1, _OutputIterator2>(__out_true, __out_false);
}

template <class _ForwardIterator, class _Predicate>
inline _ForwardIterator partition_point(_ForwardIterator __first, _ForwardIterator __last, _Predicate __pred) {
    ptrdiff_t __len = std::distance(__first, __last);
    while (__len > 0) {
        ptrdiff_t __half = __len / 2;
        _ForwardIterator __middle = std::next(__first, __half);
        if (__pred(*__middle)) {
            __first = std::next(__middle);
            __len = __len - __half - 1;
        } else {
            __len = __half;
        }
    }
    return __first;
}

template <class _BidirectionalIterator, class _Predicate>
inline _BidirectionalIterator
stable_partition(_BidirectionalIterator __first, _BidirectionalIterator __last, _Predicate __pred) {
    ptrdiff_t __len = std::distance(__first, __last);
    if (__len == 0) return __first;
    if (__len == 1) {
        return __pred(*__first) ? __last : __first;
    }
    ptrdiff_t __half = __len / 2;
    _BidirectionalIterator __middle = std::next(__first, __half);
    _BidirectionalIterator __p1 = std::stable_partition(__first, __middle, __pred);
    _BidirectionalIterator __p2 = std::stable_partition(__middle, __last, __pred);
    return std::rotate(__p1, __middle, __p2);
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_PARTITION_H_
