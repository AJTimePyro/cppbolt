#ifndef _MINI_STL_DETAIL_ALGORITHMS_MODIFYING_H_
#define _MINI_STL_DETAIL_ALGORITHMS_MODIFYING_H_

#include <utility>
#include <detail/iterator.h>
#include <detail/algorithms/comp.h>
#include <detail/algorithms/non_modifying.h>

namespace std {

template <class _InputIterator, class _OutputIterator>
inline _OutputIterator copy(_InputIterator __first, _InputIterator __last, _OutputIterator __result) {
    for (; __first != __last; ++__first, ++__result) {
        *__result = *__first;
    }
    return __result;
}

template <class _InputIterator, class _Size, class _OutputIterator>
inline _OutputIterator copy_n(_InputIterator __first, _Size __n, _OutputIterator __result) {
    for (_Size __i = 0; __i < __n; ++__i, ++__first, ++__result) {
        *__result = *__first;
    }
    return __result;
}

template <class _InputIterator, class _OutputIterator, class _Predicate>
inline _OutputIterator copy_if(_InputIterator __first, _InputIterator __last,
                               _OutputIterator __result, _Predicate __pred) {
    for (; __first != __last; ++__first) {
        if (__pred(*__first)) {
            *__result = *__first;
            ++__result;
        }
    }
    return __result;
}

template <class _BidirectionalIterator1, class _BidirectionalIterator2>
inline _BidirectionalIterator2 copy_backward(_BidirectionalIterator1 __first, _BidirectionalIterator1 __last,
                                             _BidirectionalIterator2 __result) {
    while (__first != __last) {
        --__last;
        --__result;
        *__result = *__last;
    }
    return __result;
}

template <class _InputIterator, class _OutputIterator>
inline _OutputIterator move(_InputIterator __first, _InputIterator __last, _OutputIterator __result) {
    for (; __first != __last; ++__first, ++__result) {
        *__result = std::move(*__first);
    }
    return __result;
}

template <class _BidirectionalIterator1, class _BidirectionalIterator2>
inline _BidirectionalIterator2 move_backward(_BidirectionalIterator1 __first, _BidirectionalIterator1 __last,
                                             _BidirectionalIterator2 __result) {
    while (__first != __last) {
        --__last;
        --__result;
        *__result = std::move(*__last);
    }
    return __result;
}

template <class _ForwardIterator, class _Tp>
inline void fill(_ForwardIterator __first, _ForwardIterator __last, const _Tp& __value) {
    for (; __first != __last; ++__first) {
        *__first = __value;
    }
}

template <class _OutputIterator, class _Size, class _Tp>
inline _OutputIterator fill_n(_OutputIterator __first, _Size __count, const _Tp& __value) {
    for (_Size __i = 0; __i < __count; ++__i, ++__first) {
        *__first = __value;
    }
    return __first;
}

template <class _InputIterator, class _OutputIterator, class _UnaryOperation>
inline _OutputIterator transform(_InputIterator __first, _InputIterator __last,
                                 _OutputIterator __result, _UnaryOperation __op) {
    for (; __first != __last; ++__first, ++__result) {
        *__result = __op(*__first);
    }
    return __result;
}

template <class _InputIterator1, class _InputIterator2, class _OutputIterator, class _BinaryOperation>
inline _OutputIterator transform(_InputIterator1 __first1, _InputIterator1 __last1,
                                 _InputIterator2 __first2, _OutputIterator __result,
                                 _BinaryOperation __op) {
    for (; __first1 != __last1; ++__first1, ++__first2, ++__result) {
        *__result = __op(*__first1, *__first2);
    }
    return __result;
}

template <class _ForwardIterator, class _Generator>
inline void generate(_ForwardIterator __first, _ForwardIterator __last, _Generator __gen) {
    for (; __first != __last; ++__first) {
        *__first = __gen();
    }
}

template <class _OutputIterator, class _Size, class _Generator>
inline _OutputIterator generate_n(_OutputIterator __first, _Size __count, _Generator __gen) {
    for (_Size __i = 0; __i < __count; ++__i, ++__first) {
        *__first = __gen();
    }
    return __first;
}

template <class _ForwardIterator, class _Tp>
inline _ForwardIterator remove(_ForwardIterator __first, _ForwardIterator __last, const _Tp& __value) {
    __first = std::find(__first, __last, __value);
    if (__first != __last) {
        _ForwardIterator __i = __first;
        for (++__i; __i != __last; ++__i) {
            if (!(*__i == __value)) {
                *__first = std::move(*__i);
                ++__first;
            }
        }
    }
    return __first;
}

template <class _ForwardIterator, class _Predicate>
inline _ForwardIterator remove_if(_ForwardIterator __first, _ForwardIterator __last, _Predicate __pred) {
    __first = std::find_if(__first, __last, __pred);
    if (__first != __last) {
        _ForwardIterator __i = __first;
        for (++__i; __i != __last; ++__i) {
            if (!__pred(*__i)) {
                *__first = std::move(*__i);
                ++__first;
            }
        }
    }
    return __first;
}

template <class _InputIterator, class _OutputIterator, class _Tp>
inline _OutputIterator remove_copy(_InputIterator __first, _InputIterator __last,
                                   _OutputIterator __result, const _Tp& __value) {
    for (; __first != __last; ++__first) {
        if (!(*__first == __value)) {
            *__result = *__first;
            ++__result;
        }
    }
    return __result;
}

template <class _InputIterator, class _OutputIterator, class _Predicate>
inline _OutputIterator remove_copy_if(_InputIterator __first, _InputIterator __last,
                                      _OutputIterator __result, _Predicate __pred) {
    for (; __first != __last; ++__first) {
        if (!__pred(*__first)) {
            *__result = *__first;
            ++__result;
        }
    }
    return __result;
}

template <class _ForwardIterator, class _Tp>
inline void replace(_ForwardIterator __first, _ForwardIterator __last,
                    const _Tp& __old_value, const _Tp& __new_value) {
    for (; __first != __last; ++__first) {
        if (*__first == __old_value) {
            *__first = __new_value;
        }
    }
}

template <class _ForwardIterator, class _Predicate, class _Tp>
inline void replace_if(_ForwardIterator __first, _ForwardIterator __last,
                       _Predicate __pred, const _Tp& __new_value) {
    for (; __first != __last; ++__first) {
        if (__pred(*__first)) {
            *__first = __new_value;
        }
    }
}

template <class _InputIterator, class _OutputIterator, class _Tp>
inline _OutputIterator replace_copy(_InputIterator __first, _InputIterator __last,
                                    _OutputIterator __result,
                                    const _Tp& __old_value, const _Tp& __new_value) {
    for (; __first != __last; ++__first, ++__result) {
        if (*__first == __old_value) {
            *__result = __new_value;
        } else {
            *__result = *__first;
        }
    }
    return __result;
}

template <class _InputIterator, class _OutputIterator, class _Predicate, class _Tp>
inline _OutputIterator replace_copy_if(_InputIterator __first, _InputIterator __last,
                                       _OutputIterator __result,
                                       _Predicate __pred, const _Tp& __new_value) {
    for (; __first != __last; ++__first, ++__result) {
        if (__pred(*__first)) {
            *__result = __new_value;
        } else {
            *__result = *__first;
        }
    }
    return __result;
}

template <class _ForwardIterator1, class _ForwardIterator2>
inline _ForwardIterator2 swap_ranges(_ForwardIterator1 __first1, _ForwardIterator1 __last1,
                                     _ForwardIterator2 __first2) {
    for (; __first1 != __last1; ++__first1, ++__first2) {
        std::iter_swap(__first1, __first2);
    }
    return __first2;
}

template <class _BidirectionalIterator>
inline void reverse(_BidirectionalIterator __first, _BidirectionalIterator __last) {
    if (__first == __last) return;
    --__last;
    while (__first != __last) {
        std::iter_swap(__first, __last);
        ++__first;
        if (__first == __last) break;
        --__last;
    }
}

template <class _BidirectionalIterator, class _OutputIterator>
inline _OutputIterator reverse_copy(_BidirectionalIterator __first, _BidirectionalIterator __last,
                                    _OutputIterator __result) {
    while (__first != __last) {
        --__last;
        *__result = *__last;
        ++__result;
    }
    return __result;
}

template <class _ForwardIterator>
inline _ForwardIterator rotate(_ForwardIterator __first, _ForwardIterator __middle, _ForwardIterator __last) {
    if (__first == __middle) return __last;
    if (__middle == __last) return __first;
    std::reverse(__first, __middle);
    std::reverse(__middle, __last);
    std::reverse(__first, __last);
    _ForwardIterator __ret = __first;
    for (_ForwardIterator __it = __middle; __it != __last; ++__it, ++__ret) {}
    return __ret;
}

template <class _ForwardIterator, class _OutputIterator>
inline _OutputIterator rotate_copy(_ForwardIterator __first, _ForwardIterator __middle,
                                   _ForwardIterator __last, _OutputIterator __result) {
    __result = std::copy(__middle, __last, __result);
    return std::copy(__first, __middle, __result);
}

template <class _ForwardIterator, class _BinaryPredicate>
inline _ForwardIterator unique(_ForwardIterator __first, _ForwardIterator __last, _BinaryPredicate __pred) {
    __first = std::adjacent_find(__first, __last, __pred);
    if (__first == __last) return __last;
    _ForwardIterator __dest = __first;
    while (++__first != __last) {
        if (!__pred(*__dest, *__first)) {
            *++__dest = std::move(*__first);
        }
    }
    return ++__dest;
}

template <class _ForwardIterator>
inline _ForwardIterator unique(_ForwardIterator __first, _ForwardIterator __last) {
    return std::unique(__first, __last, __equal_to());
}

template <class _InputIterator, class _OutputIterator, class _BinaryPredicate>
inline _OutputIterator unique_copy(_InputIterator __first, _InputIterator __last,
                                   _OutputIterator __result, _BinaryPredicate __pred) {
    if (__first == __last) return __result;
    auto __val = *__first;
    *__result = __val;
    ++__result;
    ++__first;
    while (__first != __last) {
        if (!__pred(__val, *__first)) {
            __val = *__first;
            *__result = __val;
            ++__result;
        }
        ++__first;
    }
    return __result;
}

template <class _InputIterator, class _OutputIterator>
inline _OutputIterator unique_copy(_InputIterator __first, _InputIterator __last,
                                   _OutputIterator __result) {
    return std::unique_copy(__first, __last, __result, __equal_to());
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_MODIFYING_H_
