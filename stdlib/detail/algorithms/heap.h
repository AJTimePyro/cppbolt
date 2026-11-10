#ifndef _MINI_STL_DETAIL_ALGORITHMS_HEAP_H_
#define _MINI_STL_DETAIL_ALGORITHMS_HEAP_H_

#include <cstddef>
#include <utility>
#include <detail/algorithms/comp.h>

namespace std {

namespace detail {

template <class _RandomAccessIterator, class _Compare>
inline void __sift_up(_RandomAccessIterator __first, ptrdiff_t __child, _Compare __comp) {
    while (__child > 0) {
        ptrdiff_t __parent = (__child - 1) / 2;
        if (__comp(*(__first + __parent), *(__first + __child))) {
            std::swap(*(__first + __parent), *(__first + __child));
            __child = __parent;
        } else {
            break;
        }
    }
}

template <class _RandomAccessIterator, class _Compare>
inline void __sift_down(_RandomAccessIterator __first, ptrdiff_t __root, ptrdiff_t __bottom, _Compare __comp) {
    while (__root * 2 + 1 <= __bottom) {
        ptrdiff_t __child = __root * 2 + 1;
        if (__child + 1 <= __bottom && __comp(*(__first + __child), *(__first + __child + 1))) {
            ++__child;
        }
        if (__comp(*(__first + __root), *(__first + __child))) {
            std::swap(*(__first + __root), *(__first + __child));
            __root = __child;
        } else {
            return;
        }
    }
}

} // namespace detail

template <class _RandomAccessIterator, class _Compare>
inline void push_heap(_RandomAccessIterator __first, _RandomAccessIterator __last, _Compare __comp) {
    ptrdiff_t __count = __last - __first;
    if (__count > 1) {
        detail::__sift_up(__first, __count - 1, __comp);
    }
}

template <class _RandomAccessIterator>
inline void push_heap(_RandomAccessIterator __first, _RandomAccessIterator __last) {
    std::push_heap(__first, __last, __less<>());
}

template <class _RandomAccessIterator, class _Compare>
inline void pop_heap(_RandomAccessIterator __first, _RandomAccessIterator __last, _Compare __comp) {
    ptrdiff_t __count = __last - __first;
    if (__count > 1) {
        std::swap(*__first, *(__last - 1));
        detail::__sift_down(__first, 0, __count - 2, __comp);
    }
}

template <class _RandomAccessIterator>
inline void pop_heap(_RandomAccessIterator __first, _RandomAccessIterator __last) {
    std::pop_heap(__first, __last, __less<>());
}

template <class _RandomAccessIterator, class _Compare>
inline void make_heap(_RandomAccessIterator __first, _RandomAccessIterator __last, _Compare __comp) {
    ptrdiff_t __count = __last - __first;
    if (__count > 1) {
        for (ptrdiff_t __i = (__count - 2) / 2; __i >= 0; --__i) {
            detail::__sift_down(__first, __i, __count - 1, __comp);
        }
    }
}

template <class _RandomAccessIterator>
inline void make_heap(_RandomAccessIterator __first, _RandomAccessIterator __last) {
    std::make_heap(__first, __last, __less<>());
}

template <class _RandomAccessIterator, class _Compare>
inline void sort_heap(_RandomAccessIterator __first, _RandomAccessIterator __last, _Compare __comp) {
    while (__last - __first > 1) {
        std::pop_heap(__first, __last, __comp);
        --__last;
    }
}

template <class _RandomAccessIterator>
inline void sort_heap(_RandomAccessIterator __first, _RandomAccessIterator __last) {
    std::sort_heap(__first, __last, __less<>());
}

template <class _RandomAccessIterator, class _Compare>
inline _RandomAccessIterator is_heap_until(_RandomAccessIterator __first, _RandomAccessIterator __last, _Compare __comp) {
    ptrdiff_t __count = __last - __first;
    for (ptrdiff_t __c = 1; __c < __count; ++__c) {
        ptrdiff_t __parent = (__c - 1) / 2;
        if (__comp(*(__first + __parent), *(__first + __c))) {
            return __first + __c;
        }
    }
    return __last;
}

template <class _RandomAccessIterator>
inline _RandomAccessIterator is_heap_until(_RandomAccessIterator __first, _RandomAccessIterator __last) {
    return std::is_heap_until(__first, __last, __less<>());
}

template <class _RandomAccessIterator, class _Compare>
inline bool is_heap(_RandomAccessIterator __first, _RandomAccessIterator __last, _Compare __comp) {
    return std::is_heap_until(__first, __last, __comp) == __last;
}

template <class _RandomAccessIterator>
inline bool is_heap(_RandomAccessIterator __first, _RandomAccessIterator __last) {
    return std::is_heap(__first, __last, __less<>());
}

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_HEAP_H_
