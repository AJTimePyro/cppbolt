#include <cstdio>
#include <cstddef>

// Multidimensional array partial-specialization deduction binds bounds in
// source order (outermost first) and keeps unbounded arrays distinct from
// fixed bounds. Verified against Clang 23.1.5 and GCC 16.2.1.

template <class T> struct S1 { static const int v = 0; static const int n = -1; };
template <class T, size_t N> struct S1<T[N]> { static const int v = 1; static const int n = (int)N; };

template <class T> struct S2 { static const int v = 0; };
template <class T> struct S2<T[]> { static const int v = 1; };

template <class T> struct S3 { static const int v = 0; static const int n = -1; static const int m = -1; };
template <class T, size_t N, size_t M> struct S3<T[N][M]> {
  static const int v = 1; static const int n = (int)N; static const int m = (int)M;
};

template <class T> struct S4 { static const int v = 0; static const int m = -1; };
template <class T, size_t M> struct S4<T[][M]> { static const int v = 1; static const int m = (int)M; };

template <class T> struct S5 { static const int v = 0; static const int n = -1; };
template <class T, size_t N> struct S5<T[N][N]> { static const int v = 1; static const int n = (int)N; };

template <class T> struct S6 { static const int v = 0; static const int n = -1; };
template <class T, size_t N> struct S6<const T[N]> { static const int v = 1; static const int n = (int)N; };

template <class T> struct O { static const int v = 0; };
template <class T, size_t N> struct O<T[N]> { static const int v = 1; };
template <class T, size_t N, size_t M> struct O<T[N][M]> { static const int v = 2; };
template <class T, size_t N, size_t M, size_t K> struct O<T[N][M][K]> { static const int v = 3; };
template <class T> struct O<T[]> { static const int v = 4; };

template <class T> struct P { static const int v = 0; };
template <class T, size_t N> struct P<T[N]> { static const int v = 1; };
template <class T> struct P<T[3]> { static const int v = 2; };

int main() {
  printf("S1 %d %d | %d\n", S1<int[3]>::v, S1<int[3]>::n, S1<int>::v);
  printf("S2 %d %d\n", S2<int[]>::v, S2<int[4]>::v);
  printf("S3 %d %d %d\n", S3<int[2][3]>::v, S3<int[2][3]>::n, S3<int[2][3]>::m);
  printf("S4 %d %d\n", S4<int[][5]>::v, S4<int[][5]>::m);
  printf("S5 %d %d | %d\n", S5<int[3][3]>::v, S5<int[3][3]>::n, S5<int[3][4]>::v);
  printf("S6 %d %d | %d\n", S6<const int[7]>::v, S6<const int[7]>::n, S6<int[7]>::v);
  printf("O1 %d O2 %d O3 %d Ounb %d Oint %d\n", O<int[2]>::v, O<int[2][3]>::v,
         O<int[2][3][4]>::v, O<int[]>::v, O<int>::v);
  printf("P3 %d P4 %d Pint %d Punb %d\n", P<int[3]>::v, P<int[4]>::v, P<int>::v, P<int[]>::v);
  return 0;
}
