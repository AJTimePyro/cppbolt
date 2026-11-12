#ifndef _MINI_STL_DETAIL_ALGORITHMS_MERGE_H_
#define _MINI_STL_DETAIL_ALGORITHMS_MERGE_H_

#include <cstddef>
#include <utility>
#include <detail/iterator.h>
#include <detail/algorithms/comp.h>
#include <detail/algorithms/modifying.h>
#include <detail/algorithms/binary_search.h>

namespace std {

template <class _InputIterator1, class _InputIterator2, class _OutputIterator, class _Compare>
inline _OutputIterator merge(_InputIterator1 __first1, _InputIterator1 __last1,
                             _InputIterator2 __first2, _InputIterator2 __last2,
                             _OutputIterator __result, _Compare __comp) {
    for (; __first1 != __last1; ++__result) {
        if (__first2 == __last2) {
            return std::copy(__first1, __last1, __result);
        }
        if (__comp(*__first2, *__first1)) {
            *__result = *__first2;
            ++__first2;
        } else {
            *__result = *__first1;
            ++__first1;
        }
    }
    return std::copy(__first2, __last2, __result);
}

template <class _InputIterator1, class _InputIterator2, class _OutputIterator>
inline _OutputIterator merge(_InputIterator1 __first1, _InputIterator1 __last1,
                             _InputIterator2 __first2, _InputIterator2 __last2,
                             _OutputIterator __result) {
    return std::merge(__first1, __last1, __first2, __last2, __result, __less<>());
}

template <class _BidirectionalIterator, class _Compare>
inline void inplace_merge(_BidirectionalIterator __first, _BidirectionalIterator __middle,
                          _BidirectionalIterator __last, _Compare __comp) {
    ptrdiff_t __len1 = std::distance(__first, __middle);
    ptrdiff_t __len2 = std::distance(__middle, __last);
    if (__len1 == 0 || __len2 == 0) return;
    if (__len1 + __len2 == 2) {
        if (__comp(*__middle, *__first)) {
            std::iter_swap(__first, __middle);
        }
        return;
    }
    _BidirectionalIterator __cut1 = __first;
    _BidirectionalIterator __cut2 = __middle;
    if (__len1 >= __len2) {
        ptrdiff_t __half1 = __len1 / 2;
        __cut1 = std::next(__first, __half1);
        __cut2 = std::lower_bound(__middle, __last, *__cut1, __comp);
    } else {
        ptrdiff_t __half2 = __len2 / 2;
        __cut2 = std::next(__middle, __half2);
        __cut1 = std::upper_bound(__first, __middle, *__cut2, __comp);
    }
    _BidirectionalIterator __new_middle = std::rotate(__cut1, __middle, __cut2);
    std::inplace_merge(__first, __cut1, __new_middle, __comp);
    std::inplace_merge(__new_middle, __cut2, __last, __comp);
}

template <class _BidirectionalIterator>
inline void inplace_merge(_BidirectionalIterator __first, _BidirectionalIterator __middle,
                          _BidirectionalIterator __last) {
    std::inplace_merge(__first, __middle, __last, __less<>());
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_MERGE_H_
