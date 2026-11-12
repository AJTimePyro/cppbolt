#ifndef _MINI_STL_DETAIL_REVERSE_ITERATOR_H_
#define _MINI_STL_DETAIL_REVERSE_ITERATOR_H_

#include <cstddef>
#include <detail/iterator.h>

namespace std {

template <class _Iter>
class reverse_iterator {
protected:
    _Iter __current;
    using _Traits = iterator_traits<_Iter>;
public:
    using iterator_type = _Iter;
    using iterator_category = typename _Traits::iterator_category;
    using difference_type = typename _Traits::difference_type;
    using value_type = typename _Traits::value_type;
    using reference = typename _Traits::reference;
    using pointer = typename _Traits::pointer;

    reverse_iterator() : __current() {}
    explicit reverse_iterator(_Iter __it) : __current(__it) {}

    _Iter base() const { return __current; }

    reference operator*() const {
        _Iter __tmp = __current;
        --__tmp;
        return *__tmp;
    }

    pointer operator->() const {
        _Iter __tmp = __current;
        --__tmp;
        return &(*__tmp);
    }

    reverse_iterator& operator++() {
        --__current;
        return *this;
    }

    reverse_iterator operator++(int) {
        reverse_iterator __tmp = *this;
        --__current;
        return __tmp;
    }

    reverse_iterator& operator--() {
        ++__current;
        return *this;
    }

    reverse_iterator operator--(int) {
        reverse_iterator __tmp = *this;
        ++__current;
        return __tmp;
    }

    bool operator==(const reverse_iterator& __y) const { return __current == __y.__current; }
    bool operator!=(const reverse_iterator& __y) const { return __current != __y.__current; }
};

} // namespace std

#endif // _MINI_STL_DETAIL_REVERSE_ITERATOR_H_
