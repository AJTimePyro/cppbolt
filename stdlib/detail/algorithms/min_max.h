#ifndef _MINI_STL_DETAIL_ALGORITHMS_MIN_MAX_H_
#define _MINI_STL_DETAIL_ALGORITHMS_MIN_MAX_H_

#include <initializer_list>
#include <utility>
#include <detail/algorithms/comp.h>

namespace std {

template <class _Tp, class _Compare>
inline const _Tp& min(const _Tp& __a, const _Tp& __b, _Compare __comp) {
    return __comp(__b, __a) ? __b : __a;
}

template <class _Tp>
inline const _Tp& min(const _Tp& __a, const _Tp& __b) {
    return (__b < __a) ? __b : __a;
}

template <class _Tp, class _Compare>
inline const _Tp& max(const _Tp& __a, const _Tp& __b, _Compare __comp) {
    return __comp(__a, __b) ? __b : __a;
}

template <class _Tp>
inline const _Tp& max(const _Tp& __a, const _Tp& __b) {
    return (__a < __b) ? __b : __a;
}

template <class _Tp, class _Compare>
inline const _Tp& clamp(const _Tp& __v, const _Tp& __lo, const _Tp& __hi, _Compare __comp) {
    return __comp(__v, __lo) ? __lo : (__comp(__hi, __v) ? __hi : __v);
}

template <class _Tp>
inline const _Tp& clamp(const _Tp& __v, const _Tp& __lo, const _Tp& __hi) {
    return (__v < __lo) ? __lo : ((__hi < __v) ? __hi : __v);
}

template <class _ForwardIterator, class _Compare>
inline _ForwardIterator min_element(_ForwardIterator __first, _ForwardIterator __last, _Compare __comp) {
    if (__first == __last) return __first;
    _ForwardIterator __min_it = __first;
    for (++__first; __first != __last; ++__first) {
        if (__comp(*__first, *__min_it)) {
            __min_it = __first;
        }
    }
    return __min_it;
}

template <class _ForwardIterator>
inline _ForwardIterator min_element(_ForwardIterator __first, _ForwardIterator __last) {
    return std::min_element(__first, __last, __less<>());
}

template <class _ForwardIterator, class _Compare>
inline _ForwardIterator max_element(_ForwardIterator __first, _ForwardIterator __last, _Compare __comp) {
    if (__first == __last) return __first;
    _ForwardIterator __max_it = __first;
    for (++__first; __first != __last; ++__first) {
        if (__comp(*__max_it, *__first)) {
            __max_it = __first;
        }
    }
    return __max_it;
}

template <class _ForwardIterator>
inline _ForwardIterator max_element(_ForwardIterator __first, _ForwardIterator __last) {
    return std::max_element(__first, __last, __less<>());
}

template <class _ForwardIterator, class _Compare>
inline pair<_ForwardIterator, _ForwardIterator>
minmax_element(_ForwardIterator __first, _ForwardIterator __last, _Compare __comp) {
    _ForwardIterator __min_it = __first;
    _ForwardIterator __max_it = __first;
    if (__first != __last) {
        for (++__first; __first != __last; ++__first) {
            if (__comp(*__first, *__min_it)) {
                __min_it = __first;
            }
            if (__comp(*__max_it, *__first)) {
                __max_it = __first;
            }
        }
    }
    return pair<_ForwardIterator, _ForwardIterator>(__min_it, __max_it);
}

template <class _ForwardIterator>
inline pair<_ForwardIterator, _ForwardIterator>
minmax_element(_ForwardIterator __first, _ForwardIterator __last) {
    return std::minmax_element(__first, __last, __less<>());
}

template <class _Tp, class _Compare>
inline pair<_Tp, _Tp> minmax(const _Tp& __a, const _Tp& __b, _Compare __comp) {
    if (__comp(__b, __a)) {
        return pair<_Tp, _Tp>(__b, __a);
    }
    return pair<_Tp, _Tp>(__a, __b);
}

template <class _Tp>
inline pair<_Tp, _Tp> minmax(const _Tp& __a, const _Tp& __b) {
    return std::minmax(__a, __b, __less<>());
}

template <class _Tp, class _Compare>
inline _Tp min(initializer_list<_Tp> __il, _Compare __comp) {
    return *std::min_element(__il.begin(), __il.end(), __comp);
}

template <class _Tp>
inline _Tp min(initializer_list<_Tp> __il) {
    return *std::min_element(__il.begin(), __il.end(), __less<>());
}

template <class _Tp, class _Compare>
inline _Tp max(initializer_list<_Tp> __il, _Compare __comp) {
    return *std::max_element(__il.begin(), __il.end(), __comp);
}

template <class _Tp>
inline _Tp max(initializer_list<_Tp> __il) {
    return *std::max_element(__il.begin(), __il.end(), __less<>());
}

template <class _Tp, class _Compare>
inline pair<_Tp, _Tp> minmax(initializer_list<_Tp> __il, _Compare __comp) {
    auto __p = std::minmax_element(__il.begin(), __il.end(), __comp);
    return pair<_Tp, _Tp>(*__p.first, *__p.second);
}

template <class _Tp>
inline pair<_Tp, _Tp> minmax(initializer_list<_Tp> __il) {
    return std::minmax(__il, __less<>());
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_MIN_MAX_H_
