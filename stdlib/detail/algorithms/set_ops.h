#ifndef _MINI_STL_DETAIL_ALGORITHMS_SET_OPS_H_
#define _MINI_STL_DETAIL_ALGORITHMS_SET_OPS_H_

#include <utility>
#include <detail/iterator.h>
#include <detail/algorithms/comp.h>
#include <detail/algorithms/modifying.h>

namespace std {

template <class _InputIterator1, class _InputIterator2, class _Compare>
inline bool includes(_InputIterator1 __first1, _InputIterator1 __last1,
                     _InputIterator2 __first2, _InputIterator2 __last2,
                     _Compare __comp) {
    for (; __first2 != __last2; ++__first1) {
        if (__first1 == __last1 || __comp(*__first2, *__first1)) return false;
        if (!__comp(*__first1, *__first2)) ++__first2;
    }
    return true;
}

template <class _InputIterator1, class _InputIterator2>
inline bool includes(_InputIterator1 __first1, _InputIterator1 __last1,
                     _InputIterator2 __first2, _InputIterator2 __last2) {
    return std::includes(__first1, __last1, __first2, __last2, __less<>());
}

template <class _InputIterator1, class _InputIterator2, class _OutputIterator, class _Compare>
inline _OutputIterator set_union(_InputIterator1 __first1, _InputIterator1 __last1,
                                 _InputIterator2 __first2, _InputIterator2 __last2,
                                 _OutputIterator __result, _Compare __comp) {
    for (; __first1 != __last1; ++__result) {
        if (__first2 == __last2) return std::copy(__first1, __last1, __result);
        if (__comp(*__first2, *__first1)) {
            *__result = *__first2;
            ++__first2;
        } else {
            *__result = *__first1;
            if (!__comp(*__first1, *__first2)) ++__first2;
            ++__first1;
        }
    }
    return std::copy(__first2, __last2, __result);
}

template <class _InputIterator1, class _InputIterator2, class _OutputIterator>
inline _OutputIterator set_union(_InputIterator1 __first1, _InputIterator1 __last1,
                                 _InputIterator2 __first2, _InputIterator2 __last2,
                                 _OutputIterator __result) {
    return std::set_union(__first1, __last1, __first2, __last2, __result, __less<>());
}

template <class _InputIterator1, class _InputIterator2, class _OutputIterator, class _Compare>
inline _OutputIterator set_intersection(_InputIterator1 __first1, _InputIterator1 __last1,
                                        _InputIterator2 __first2, _InputIterator2 __last2,
                                        _OutputIterator __result, _Compare __comp) {
    while (__first1 != __last1 && __first2 != __last2) {
        if (__comp(*__first1, *__first2)) {
            ++__first1;
        } else {
            if (!__comp(*__first2, *__first1)) {
                *__result = *__first1;
                ++__result;
                ++__first1;
            }
            ++__first2;
        }
    }
    return __result;
}

template <class _InputIterator1, class _InputIterator2, class _OutputIterator>
inline _OutputIterator set_intersection(_InputIterator1 __first1, _InputIterator1 __last1,
                                        _InputIterator2 __first2, _InputIterator2 __last2,
                                        _OutputIterator __result) {
    return std::set_intersection(__first1, __last1, __first2, __last2, __result, __less<>());
}

template <class _InputIterator1, class _InputIterator2, class _OutputIterator, class _Compare>
inline _OutputIterator set_difference(_InputIterator1 __first1, _InputIterator1 __last1,
                                      _InputIterator2 __first2, _InputIterator2 __last2,
                                      _OutputIterator __result, _Compare __comp) {
    while (__first1 != __last1) {
        if (__first2 == __last2) return std::copy(__first1, __last1, __result);
        if (__comp(*__first1, *__first2)) {
            *__result = *__first1;
            ++__result;
            ++__first1;
        } else {
            if (!__comp(*__first2, *__first1)) {
                ++__first1;
            }
            ++__first2;
        }
    }
    return __result;
}

template <class _InputIterator1, class _InputIterator2, class _OutputIterator>
inline _OutputIterator set_difference(_InputIterator1 __first1, _InputIterator1 __last1,
                                      _InputIterator2 __first2, _InputIterator2 __last2,
                                      _OutputIterator __result) {
    return std::set_difference(__first1, __last1, __first2, __last2, __result, __less<>());
}

template <class _InputIterator1, class _InputIterator2, class _OutputIterator, class _Compare>
inline _OutputIterator set_symmetric_difference(_InputIterator1 __first1, _InputIterator1 __last1,
                                                _InputIterator2 __first2, _InputIterator2 __last2,
                                                _OutputIterator __result, _Compare __comp) {
    while (__first1 != __last1) {
        if (__first2 == __last2) return std::copy(__first1, __last1, __result);
        if (__comp(*__first1, *__first2)) {
            *__result = *__first1;
            ++__result;
            ++__first1;
        } else {
            if (__comp(*__first2, *__first1)) {
                *__result = *__first2;
                ++__result;
            } else {
                ++__first1;
            }
            ++__first2;
        }
    }
    return std::copy(__first2, __last2, __result);
}

template <class _InputIterator1, class _InputIterator2, class _OutputIterator>
inline _OutputIterator set_symmetric_difference(_InputIterator1 __first1, _InputIterator1 __last1,
                                                _InputIterator2 __first2, _InputIterator2 __last2,
                                                _OutputIterator __result) {
    return std::set_symmetric_difference(__first1, __last1, __first2, __last2, __result, __less<>());
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_SET_OPS_H_
