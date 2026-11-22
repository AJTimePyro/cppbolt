#include <cstdio>
#include <type_traits>

struct S { int m; int f(); int g() const; int h() noexcept; };

// Qualifiers are part of a member-function-pointer type (finding 1).
template <class T> struct IsConstMem { static const int value = 0; };
template <class R, class C, class... A>
struct IsConstMem<R (C::*)(A...) const> { static const int value = 1; };

template <class T> struct IsNoexMem { static const int value = 0; };
template <class R, class C, class... A>
struct IsNoexMem<R (C::*)(A...) noexcept> { static const int value = 1; };

// A written bound must not behave like the unbounded `T[]` form (finding 7).
template <class T> struct ZeroBound { static const int value = 0; };
template <class T> struct ZeroBound<T[0]> { static const int value = 1; };

template <class T> struct ThreeBound { static const int value = 0; };
template <class T> struct ThreeBound<T[3]> { static const int value = 1; };

int main() {
  bool qualifiers = IsConstMem<decltype(&S::f)>::value == 0
                 && IsConstMem<decltype(&S::g)>::value == 1
                 && IsNoexMem<decltype(&S::f)>::value == 0
                 && IsNoexMem<decltype(&S::h)>::value == 1;

  bool members = std::is_member_object_pointer<int S::*>::value
              && !std::is_member_object_pointer<int (S::*)()>::value
              && std::is_member_function_pointer<int (S::*)()>::value
              && std::is_member_function_pointer<int (S::*)() const>::value
              && std::is_member_function_pointer<int (S::*)() noexcept>::value
              && std::is_member_pointer<int (S::*)()>::value;

  bool cvptr = std::is_pointer<int* const>::value
            && std::is_member_pointer<int S::* const>::value
            && std::is_member_object_pointer<int S::* const>::value;

  bool bounds = ZeroBound<int[3]>::value == 0
             && ThreeBound<int[3]>::value == 1
             && ThreeBound<int[4]>::value == 0
             && std::is_array<int[]>::value
             && std::is_array<int[3]>::value;

  std::printf("%d %d %d %d\n", (int)qualifiers, (int)members, (int)cvptr, (int)bounds);
  return 0;
}
