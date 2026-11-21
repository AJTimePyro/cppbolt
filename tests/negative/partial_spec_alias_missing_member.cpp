template <class T> using Id = T;
template <class T, class U> struct C { typedef int type; static const int which = 0; };
template <class T> struct C<Id<T>, int> { static const int which = 1; };

int main() {
  C<int, int>::type x = 0;
  return x;
}
