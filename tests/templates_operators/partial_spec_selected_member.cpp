#include <cstdio>
#include <utility>

// Nested-member validation must inspect the definition normal instantiation
// selects, not the primary or a structurally matching rejected candidate.
template <class T> struct Box { typedef T type; };
struct Yes { int get() const { return 7; } };
struct No {};

// The primary has `type`; the confirmed decltype specialization does not.
template <class T, class U> struct A { typedef int type; static const int which = 0; };
template <class T> struct A<T, typename Box<decltype(std::declval<T>().get())>::type> {
  static const int which = 1;
};

// The primary has no `type`; the confirmed specialization does.
template <class T, class U> struct B { static const int which = 0; };
template <class T> struct B<T, typename Box<decltype(T())>::type> {
  typedef T type;
  static const int which = 1;
};

// An alias-template pattern also selects its specialization.
template <class T> using Id = T;
template <class T, class U> struct C { typedef int type; static const int which = 0; };
template <class T> struct C<Id<T>, int> { static const int which = 1; };

// SFINAE depends on the selected definition: A<Yes,int> has no `type`.
template <class T> typename A<T, int>::type probe(T, int) { return 1; }
template <class T> long probe(T, double) { return -1; }

int main() {
  printf("which %d %d %d %d\n", A<Yes, int>::which, A<No, int>::which,
         B<int, int>::which, C<int, int>::which);
  A<No, int>::type a = 3;
  B<int, int>::type b = 4;
  printf("types %d %d probe %d %d\n", a, b, (int)probe(Yes(), 0), (int)probe(No(), 0));
  return 0;
}
