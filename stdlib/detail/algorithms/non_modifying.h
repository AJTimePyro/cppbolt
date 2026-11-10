#ifndef _MINI_STL_DETAIL_ALGORITHMS_NON_MODIFYING_H_
#define _MINI_STL_DETAIL_ALGORITHMS_NON_MODIFYING_H_

#include <cstddef>
#include <utility>
#include <detail/iterator.h>
#include <detail/algorithms/comp.h>

namespace std {

template <class _InputIterator, class _Predicate>
inline bool all_of(_InputIterator __first, _InputIterator __last, _Predicate __pred) {
    for (; __first != __last; ++__first) {
        if (!__pred(*__first)) return false;
    }
    return true;
}

template <class _InputIterator, class _Predicate>
inline bool any_of(_InputIterator __first, _InputIterator __last, _Predicate __pred) {
    for (; __first != __last; ++__first) {
        if (__pred(*__first)) return true;
    }
    return false;
}

template <class _InputIterator, class _Predicate>
inline bool none_of(_InputIterator __first, _InputIterator __last, _Predicate __pred) {
    for (; __first != __last; ++__first) {
        if (__pred(*__first)) return false;
    }
    return true;
}

template <class _InputIterator, class _Function>
inline _Function for_each(_InputIterator __first, _InputIterator __last, _Function __f) {
    for (; __first != __last; ++__first) {
        __f(*__first);
    }
    return __f;
}

template <class _InputIterator, class _Size, class _Function>
inline _InputIterator for_each_n(_InputIterator __first, _Size __n, _Function __f) {
    for (_Size __i = 0; __i < __n; ++__i, ++__first) {
        __f(*__first);
    }
    return __first;
}

template <class _InputIterator, class _Tp>
inline ptrdiff_t count(_InputIterator __first, _InputIterator __last, const _Tp& __value) {
    ptrdiff_t __r = 0;
    for (; __first != __last; ++__first) {
        if (*__first == __value) ++__r;
    }
    return __r;
}

template <class _InputIterator, class _Predicate>
inline ptrdiff_t count_if(_InputIterator __first, _InputIterator __last, _Predicate __pred) {
    ptrdiff_t __r = 0;
    for (; __first != __last; ++__first) {
        if (__pred(*__first)) ++__r;
    }
    return __r;
}

template <class _InputIterator1, class _InputIterator2, class _BinaryPredicate>
inline pair<_InputIterator1, _InputIterator2>
mismatch(_InputIterator1 __first1, _InputIterator1 __last1,
         _InputIterator2 __first2, _InputIterator2 __last2,
         _BinaryPredicate __pred) {
    while (__first1 != __last1 && __first2 != __last2 && __pred(*__first1, *__first2)) {
        ++__first1;
        ++__first2;
    }
    return pair<_InputIterator1, _InputIterator2>(__first1, __first2);
}

template <class _InputIterator1, class _InputIterator2>
inline pair<_InputIterator1, _InputIterator2>
mismatch(_InputIterator1 __first1, _InputIterator1 __last1,
         _InputIterator2 __first2, _InputIterator2 __last2) {
    return std::mismatch(__first1, __last1, __first2, __last2, __equal_to());
}

template <class _InputIterator1, class _InputIterator2, class _BinaryPredicate>
inline pair<_InputIterator1, _InputIterator2>
mismatch(_InputIterator1 __first1, _InputIterator1 __last1,
         _InputIterator2 __first2, _BinaryPredicate __pred) {
    while (__first1 != __last1 && __pred(*__first1, *__first2)) {
        ++__first1;
        ++__first2;
    }
    return pair<_InputIterator1, _InputIterator2>(__first1, __first2);
}

template <class _InputIterator1, class _InputIterator2>
inline pair<_InputIterator1, _InputIterator2>
mismatch(_InputIterator1 __first1, _InputIterator1 __last1, _InputIterator2 __first2) {
    return std::mismatch(__first1, __last1, __first2, __equal_to());
}

template <class _InputIterator, class _Tp>
inline _InputIterator find(_InputIterator __first, _InputIterator __last, const _Tp& __value) {
    for (; __first != __last; ++__first) {
        if (*__first == __value) return __first;
    }
    return __last;
}

template <class _InputIterator, class _Predicate>
inline _InputIterator find_if(_InputIterator __first, _InputIterator __last, _Predicate __pred) {
    for (; __first != __last; ++__first) {
        if (__pred(*__first)) return __first;
    }
    return __last;
}

template <class _InputIterator, class _Predicate>
inline _InputIterator find_if_not(_InputIterator __first, _InputIterator __last, _Predicate __pred) {
    for (; __first != __last; ++__first) {
        if (!__pred(*__first)) return __first;
    }
    return __last;
}

template <class _ForwardIterator1, class _ForwardIterator2, class _BinaryPredicate>
inline _ForwardIterator1 search(_ForwardIterator1 __first1, _ForwardIterator1 __last1,
                               _ForwardIterator2 __first2, _ForwardIterator2 __last2,
                               _BinaryPredicate __pred) {
    if (__first2 == __last2) return __first1;
    for (; __first1 != __last1; ++__first1) {
        _ForwardIterator1 __it1 = __first1;
        _ForwardIterator2 __it2 = __first2;
        while (true) {
            if (__it2 == __last2) return __first1;
            if (__it1 == __last1) return __last1;
            if (!__pred(*__it1, *__it2)) break;
            ++__it1;
            ++__it2;
        }
    }
    return __last1;
}

template <class _ForwardIterator1, class _ForwardIterator2>
inline _ForwardIterator1 search(_ForwardIterator1 __first1, _ForwardIterator1 __last1,
                               _ForwardIterator2 __first2, _ForwardIterator2 __last2) {
    return std::search(__first1, __last1, __first2, __last2, __equal_to());
}

template <class _ForwardIterator1, class _ForwardIterator2, class _BinaryPredicate>
inline _ForwardIterator1 find_end(_ForwardIterator1 __first1, _ForwardIterator1 __last1,
                                 _ForwardIterator2 __first2, _ForwardIterator2 __last2,
                                 _BinaryPredicate __pred) {
    if (__first2 == __last2) return __last1;
    _ForwardIterator1 __result = __last1;
    while (true) {
        _ForwardIterator1 __new_result = std::search(__first1, __last1, __first2, __last2, __pred);
        if (__new_result == __last1) return __result;
        __result = __new_result;
        __first1 = __result;
        ++__first1;
    }
}

template <class _ForwardIterator1, class _ForwardIterator2>
inline _ForwardIterator1 find_end(_ForwardIterator1 __first1, _ForwardIterator1 __last1,
                                 _ForwardIterator2 __first2, _ForwardIterator2 __last2) {
    return std::find_end(__first1, __last1, __first2, __last2, __equal_to());
}

template <class _InputIterator, class _ForwardIterator, class _BinaryPredicate>
inline _InputIterator find_first_of(_InputIterator __first1, _InputIterator __last1,
                                   _ForwardIterator __first2, _ForwardIterator __last2,
                                   _BinaryPredicate __pred) {
    for (; __first1 != __last1; ++__first1) {
        for (_ForwardIterator __it = __first2; __it != __last2; ++__it) {
            if (__pred(*__first1, *__it)) return __first1;
        }
    }
    return __last1;
}

template <class _InputIterator, class _ForwardIterator>
inline _InputIterator find_first_of(_InputIterator __first1, _InputIterator __last1,
                                   _ForwardIterator __first2, _ForwardIterator __last2) {
    return std::find_first_of(__first1, __last1, __first2, __last2, __equal_to());
}

template <class _ForwardIterator, class _BinaryPredicate>
inline _ForwardIterator adjacent_find(_ForwardIterator __first, _ForwardIterator __last, _BinaryPredicate __pred) {
    if (__first != __last) {
        _ForwardIterator __next = __first;
        for (++__next; __next != __last; ++__first, ++__next) {
            if (__pred(*__first, *__next)) return __first;
        }
    }
    return __last;
}

template <class _ForwardIterator>
inline _ForwardIterator adjacent_find(_ForwardIterator __first, _ForwardIterator __last) {
    return std::adjacent_find(__first, __last, __equal_to());
}

template <class _ForwardIterator, class _Size, class _Tp, class _BinaryPredicate>
inline _ForwardIterator search_n(_ForwardIterator __first, _ForwardIterator __last,
                                _Size __count, const _Tp& __value, _BinaryPredicate __pred) {
    if (__count <= 0) return __first;
    for (; __first != __last; ++__first) {
        if (!__pred(*__first, __value)) continue;
        _ForwardIterator __candidate = __first;
        _Size __n = 1;
        while (++__first != __last) {
            if (!__pred(*__first, __value)) break;
            ++__n;
            if (__n == __count) return __candidate;
        }
        if (__first == __last) return __last;
    }
    return __last;
}

template <class _ForwardIterator, class _Size, class _Tp>
inline _ForwardIterator search_n(_ForwardIterator __first, _ForwardIterator __last,
                                _Size __count, const _Tp& __value) {
    return std::search_n(__first, __last, __count, __value, __equal_to());
}

template <class _InputIterator1, class _InputIterator2, class _BinaryPredicate>
inline bool equal(_InputIterator1 __first1, _InputIterator1 __last1,
                  _InputIterator2 __first2, _InputIterator2 __last2,
                  _BinaryPredicate __pred) {
    for (; __first1 != __last1 && __first2 != __last2; ++__first1, ++__first2) {
        if (!__pred(*__first1, *__first2)) return false;
    }
    return __first1 == __last1 && __first2 == __last2;
}

template <class _InputIterator1, class _InputIterator2>
inline bool equal(_InputIterator1 __first1, _InputIterator1 __last1,
                  _InputIterator2 __first2, _InputIterator2 __last2) {
    return std::equal(__first1, __last1, __first2, __last2, __equal_to());
}

template <class _InputIterator1, class _InputIterator2, class _BinaryPredicate>
inline bool equal(_InputIterator1 __first1, _InputIterator1 __last1,
                  _InputIterator2 __first2, _BinaryPredicate __pred) {
    for (; __first1 != __last1; ++__first1, ++__first2) {
        if (!__pred(*__first1, *__first2)) return false;
    }
    return true;
}

template <class _InputIterator1, class _InputIterator2>
inline bool equal(_InputIterator1 __first1, _InputIterator1 __last1, _InputIterator2 __first2) {
    return std::equal(__first1, __last1, __first2, __equal_to());
}

template <class _InputIterator1, class _InputIterator2, class _Compare>
inline bool lexicographical_compare(_InputIterator1 __first1, _InputIterator1 __last1,
                                    _InputIterator2 __first2, _InputIterator2 __last2,
                                    _Compare __comp) {
    for (; __first1 != __last1 && __first2 != __last2; ++__first1, ++__first2) {
        if (__comp(*__first1, *__first2)) return true;
        if (__comp(*__first2, *__first1)) return false;
    }
    return __first1 == __last1 && __first2 != __last2;
}

template <class _InputIterator1, class _InputIterator2>
inline bool lexicographical_compare(_InputIterator1 __first1, _InputIterator1 __last1,
                                    _InputIterator2 __first2, _InputIterator2 __last2) {
    return std::lexicographical_compare(__first1, __last1, __first2, __last2, __less<>());
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_NON_MODIFYING_H_
