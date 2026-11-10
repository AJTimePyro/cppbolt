#ifndef _MINI_STL_DETAIL_REVERSE_ITERATOR_H_
#define _MINI_STL_DETAIL_REVERSE_ITERATOR_H_

namespace std {

template <class _Iter, class _Tp = int>
class reverse_iterator {
protected:
    _Iter __current;
public:
    using iterator_type = _Iter;
    using value_type = _Tp;
    using reference = _Tp&;
    using pointer = _Tp*;

    reverse_iterator() : __current() {}
    explicit reverse_iterator(_Iter __it) : __current(__it) {}

    _Iter base() const { return __current; }

    _Tp& operator*() const {
        _Iter __tmp = __current;
        --__tmp;
        return *__tmp;
    }

    _Iter operator->() const {
        _Iter __tmp = __current;
        --__tmp;
        return __tmp;
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
