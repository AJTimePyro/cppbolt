struct NoType {};
template <class T> struct Maybe { typedef T type; };
template <> struct Maybe<NoType> {};

using Invalid = Maybe<NoType>::type;
int main() { return 0; }
