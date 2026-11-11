#ifndef _MINI_STL_DETAIL_ITERATOR_H_
#define _MINI_STL_DETAIL_ITERATOR_H_

#include <cstddef>
#include <utility>

namespace std {

struct input_iterator_tag {};
struct output_iterator_tag {};
struct forward_iterator_tag : input_iterator_tag {};
struct bidirectional_iterator_tag : forward_iterator_tag {};
struct random_access_iterator_tag : bidirectional_iterator_tag {};

template <class _Iterator>
struct iterator_traits {
    typedef typename _Iterator::iterator_category iterator_category;
    typedef typename _Iterator::difference_type difference_type;
    typedef typename _Iterator::value_type      value_type;
    typedef typename _Iterator::pointer         pointer;
    typedef typename _Iterator::reference       reference;
};

template <class _Tp>
struct iterator_traits<_Tp*> {
    typedef random_access_iterator_tag iterator_category;
    typedef ptrdiff_t difference_type;
    typedef _Tp       value_type;
    typedef _Tp*      pointer;
    typedef _Tp&      reference;
};

template <class _Tp>
struct iterator_traits<const _Tp*> {
    typedef random_access_iterator_tag iterator_category;
    typedef ptrdiff_t  difference_type;
    typedef _Tp        value_type;
    typedef const _Tp* pointer;
    typedef const _Tp& reference;
};

namespace detail {

template <class _InputIterator>
inline ptrdiff_t distance_impl(_InputIterator __first, _InputIterator __last,
                               input_iterator_tag) {
    ptrdiff_t __n = 0;
    for (; __first != __last; ++__first) {
        ++__n;
    }
    return __n;
}

template <class _ForwardIterator>
inline ptrdiff_t distance_impl(_ForwardIterator __first, _ForwardIterator __last,
                               forward_iterator_tag) {
    ptrdiff_t __n = 0;
    for (; __first != __last; ++__first) {
        ++__n;
    }
    return __n;
}

template <class _BidirectionalIterator>
inline ptrdiff_t distance_impl(_BidirectionalIterator __first, _BidirectionalIterator __last,
                               bidirectional_iterator_tag) {
    ptrdiff_t __n = 0;
    for (; __first != __last; ++__first) {
        ++__n;
    }
    return __n;
}

template <class _RandomAccessIterator>
inline ptrdiff_t distance_impl(_RandomAccessIterator __first, _RandomAccessIterator __last,
                               random_access_iterator_tag) {
    return __last - __first;
}

template <class _InputIterator, class _Distance>
inline void advance_impl(_InputIterator& __i, _Distance __n, input_iterator_tag) {
    while (__n > 0) {
        ++__i;
        --__n;
    }
}

template <class _ForwardIterator, class _Distance>
inline void advance_impl(_ForwardIterator& __i, _Distance __n, forward_iterator_tag) {
    while (__n > 0) {
        ++__i;
        --__n;
    }
}

template <class _BidirectionalIterator, class _Distance>
inline void advance_impl(_BidirectionalIterator& __i, _Distance __n, bidirectional_iterator_tag) {
    while (__n > 0) {
        ++__i;
        --__n;
    }
    while (__n < 0) {
        --__i;
        ++__n;
    }
}

template <class _RandomAccessIterator, class _Distance>
inline void advance_impl(_RandomAccessIterator& __i, _Distance __n, random_access_iterator_tag) {
    __i += __n;
}

} // namespace detail

template <class _Tp>
inline ptrdiff_t distance(_Tp* __first, _Tp* __last) {
    return __last - __first;
}

template <class _Tp>
inline ptrdiff_t distance(const _Tp* __first, const _Tp* __last) {
    return __last - __first;
}

template <class _InputIterator>
inline ptrdiff_t distance(_InputIterator __first, _InputIterator __last) {
    ptrdiff_t __n = 0;
    while (__first != __last) {
        ++__first;
        ++__n;
    }
    return __n;
}

template <class _Tp, class _Distance>
inline void advance(_Tp*& __i, _Distance __n) {
    __i += __n;
}

template <class _Tp, class _Distance>
inline void advance(const _Tp*& __i, _Distance __n) {
    __i += __n;
}

template <class _InputIterator, class _Distance>
inline void advance(_InputIterator& __i, _Distance __n) {
    while (__n > 0) {
        ++__i;
        --__n;
    }
    while (__n < 0) {
        --__i;
        ++__n;
    }
}

template <class _InputIterator>
inline _InputIterator next(_InputIterator __x, ptrdiff_t __n = 1) {
    std::advance(__x, __n);
    return __x;
}

template <class _Tp>
inline _Tp* prev(_Tp* __x, ptrdiff_t __n = 1) {
    return __x - __n;
}

template <class _Tp>
inline const _Tp* prev(const _Tp* __x, ptrdiff_t __n = 1) {
    return __x - __n;
}

template <class _BidirectionalIterator>
inline _BidirectionalIterator prev(_BidirectionalIterator __x, ptrdiff_t __n = 1) {
    std::advance(__x, -__n);
    return __x;
}

template <class _ForwardIterator1, class _ForwardIterator2>
inline void iter_swap(_ForwardIterator1 __a, _ForwardIterator2 __b) {
    std::swap(*__a, *__b);
}

} // namespace std

#endif // _MINI_STL_DETAIL_ITERATOR_H_
