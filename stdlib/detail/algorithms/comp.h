#ifndef _MINI_STL_DETAIL_ALGORITHMS_COMP_H_
#define _MINI_STL_DETAIL_ALGORITHMS_COMP_H_

namespace std {

template <class _Tp = void>
struct equal_to {
    template <class _T1, class _T2>
    constexpr bool operator()(const _T1& __x, const _T2& __y) const {
        return __x == __y;
    }
};

template <class _Tp = void>
struct less {
    template <class _T1, class _T2>
    constexpr bool operator()(const _T1& __lhs, const _T2& __rhs) const {
        return __lhs < __rhs;
    }
};

template <class _Tp = void>
struct greater {
    template <class _T1, class _T2>
    constexpr bool operator()(const _T1& __lhs, const _T2& __rhs) const {
        return __lhs > __rhs;
    }
};

// Backwards-compatible internal names for the public comparators.
struct __equal_to {
    template <class _T1, class _T2>
    constexpr bool operator()(const _T1& __x, const _T2& __y) const {
        return __x == __y;
    }
};

template <class _T1 = void, class _T2 = _T1>
struct __less {
    template <class _Tp, class _Up>
    constexpr bool operator()(const _Tp& __lhs, const _Up& __rhs) const {
        return __lhs < __rhs;
    }
};

template <class _T1 = void, class _T2 = _T1>
struct __greater {
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
