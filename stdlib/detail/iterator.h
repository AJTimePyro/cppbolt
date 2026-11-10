#ifndef _MINI_STL_DETAIL_ITERATOR_H_
#define _MINI_STL_DETAIL_ITERATOR_H_

#include <cstddef>
#include <utility>

namespace std {

template <class _Iterator>
struct iterator_traits {
    typedef typename _Iterator::difference_type difference_type;
    typedef typename _Iterator::value_type      value_type;
    typedef typename _Iterator::pointer         pointer;
    typedef typename _Iterator::reference       reference;
};

template <class _Tp>
struct iterator_traits<_Tp*> {
    typedef ptrdiff_t difference_type;
    typedef _Tp       value_type;
    typedef _Tp*      pointer;
    typedef _Tp&      reference;
};

template <class _Tp>
struct iterator_traits<const _Tp*> {
    typedef ptrdiff_t  difference_type;
    typedef _Tp        value_type;
    typedef const _Tp* pointer;
    typedef const _Tp& reference;
};

template <class _InputIterator>
inline ptrdiff_t distance(_InputIterator __first, _InputIterator __last) {
    return __last - __first;
}

template <class _InputIterator, class _Distance>
inline void advance(_InputIterator& __i, _Distance __n) {
    __i += __n;
}

template <class _InputIterator>
inline _InputIterator next(_InputIterator __x, ptrdiff_t __n = 1) {
    return __x + __n;
}

template <class _BidirectionalIterator>
inline _BidirectionalIterator prev(_BidirectionalIterator __x, ptrdiff_t __n = 1) {
    return __x - __n;
}

template <class _ForwardIterator1, class _ForwardIterator2>
inline void iter_swap(_ForwardIterator1 __a, _ForwardIterator2 __b) {
    std::swap(*__a, *__b);
}

} // namespace std

#endif // _MINI_STL_DETAIL_ITERATOR_H_
