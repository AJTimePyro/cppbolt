#ifndef _MINI_STL_DETAIL_ALGORITHMS_PERMUTATION_H_
#define _MINI_STL_DETAIL_ALGORITHMS_PERMUTATION_H_

#include <cstddef>
#include <utility>
#include <detail/iterator.h>
#include <detail/algorithms/comp.h>
#include <detail/algorithms/modifying.h>

namespace std {

template <class _ForwardIterator1, class _ForwardIterator2, class _BinaryPredicate>
inline bool is_permutation(_ForwardIterator1 __first1, _ForwardIterator1 __last1,
                           _ForwardIterator2 __first2, _ForwardIterator2 __last2,
                           _BinaryPredicate __pred) {
    for (; __first1 != __last1 && __first2 != __last2; ++__first1, ++__first2) {
        if (!__pred(*__first1, *__first2)) break;
    }
    if (__first1 == __last1 && __first2 == __last2) return true;
    if (std::distance(__first1, __last1) != std::distance(__first2, __last2)) return false;
    for (_ForwardIterator1 __i = __first1; __i != __last1; ++__i) {
        bool __already = false;
        for (_ForwardIterator1 __j = __first1; __j != __i; ++__j) {
            if (__pred(*__j, *__i)) {
                __already = true;
                break;
            }
        }
        if (__already) continue;
        ptrdiff_t __c1 = 0;
        for (_ForwardIterator1 __k = __i; __k != __last1; ++__k) {
            if (__pred(*__k, *__i)) ++__c1;
        }
        ptrdiff_t __c2 = 0;
        for (_ForwardIterator2 __k = __first2; __k != __last2; ++__k) {
            if (__pred(*__k, *__i)) ++__c2;
        }
        if (__c1 != __c2) return false;
    }
    return true;
}

template <class _ForwardIterator1, class _ForwardIterator2>
inline bool is_permutation(_ForwardIterator1 __first1, _ForwardIterator1 __last1,
                           _ForwardIterator2 __first2, _ForwardIterator2 __last2) {
    return std::is_permutation(__first1, __last1, __first2, __last2, __equal_to());
}

template <class _ForwardIterator1, class _ForwardIterator2, class _BinaryPredicate>
inline bool is_permutation(_ForwardIterator1 __first1, _ForwardIterator1 __last1,
                           _ForwardIterator2 __first2, _BinaryPredicate __pred) {
    return std::is_permutation(__first1, __last1, __first2, std::next(__first2, std::distance(__first1, __last1)), __pred);
}

template <class _ForwardIterator1, class _ForwardIterator2>
inline bool is_permutation(_ForwardIterator1 __first1, _ForwardIterator1 __last1,
                           _ForwardIterator2 __first2) {
    return std::is_permutation(__first1, __last1, __first2, __equal_to());
}

template <class _BidirectionalIterator, class _Compare>
inline bool next_permutation(_BidirectionalIterator __first, _BidirectionalIterator __last, _Compare __comp) {
    if (__first == __last) return false;
    _BidirectionalIterator __i = __last;
    if (__first == --__i) return false;
    while (true) {
        _BidirectionalIterator __i1 = __i;
        if (__comp(*--__i, *__i1)) {
            _BidirectionalIterator __j = __last;
            while (!__comp(*__i, *--__j)) {}
            std::iter_swap(__i, __j);
            std::reverse(__i1, __last);
            return true;
        }
        if (__i == __first) {
            std::reverse(__first, __last);
            return false;
        }
    }
}

template <class _BidirectionalIterator>
inline bool next_permutation(_BidirectionalIterator __first, _BidirectionalIterator __last) {
    return std::next_permutation(__first, __last, __less<>());
}

template <class _BidirectionalIterator, class _Compare>
inline bool prev_permutation(_BidirectionalIterator __first, _BidirectionalIterator __last, _Compare __comp) {
    if (__first == __last) return false;
    _BidirectionalIterator __i = __last;
    if (__first == --__i) return false;
    while (true) {
        _BidirectionalIterator __i1 = __i;
        if (__comp(*__i1, *--__i)) {
            _BidirectionalIterator __j = __last;
            while (!__comp(*--__j, *__i)) {}
            std::iter_swap(__i, __j);
            std::reverse(__i1, __last);
            return true;
        }
        if (__i == __first) {
            std::reverse(__first, __last);
            return false;
        }
    }
}

template <class _BidirectionalIterator>
inline bool prev_permutation(_BidirectionalIterator __first, _BidirectionalIterator __last) {
    return std::prev_permutation(__first, __last, __less<>());
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_PERMUTATION_H_
