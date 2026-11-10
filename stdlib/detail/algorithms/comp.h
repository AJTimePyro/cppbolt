#ifndef _MINI_STL_DETAIL_ALGORITHMS_COMP_H_
#define _MINI_STL_DETAIL_ALGORITHMS_COMP_H_

namespace std {

struct __equal_to {
    template <class _T1, class _T2>
    constexpr bool operator()(const _T1& __x, const _T2& __y) const {
        return __x == __y;
    }
};

template <class _T1 = void, class _T2 = _T1>
struct __less {};

template <>
struct __less<void, void> {
    template <class _Tp, class _Up>
    constexpr bool operator()(const _Tp& __lhs, const _Up& __rhs) const {
        return __lhs < __rhs;
    }
};

template <class _T1 = void, class _T2 = _T1>
struct __greater {};

template <>
struct __greater<void, void> {
    template <class _Tp, class _Up>
    constexpr bool operator()(const _Tp& __lhs, const _Up& __rhs) const {
        return __lhs > __rhs;
    }
};

namespace detail {
    struct DefaultLess {
        template <typename T>
        bool operator()(const T& a, const T& b) const {
            return a < b;
        }
    };
} // namespace detail

} // namespace std

#endif // _MINI_STL_DETAIL_ALGORITHMS_COMP_H_
