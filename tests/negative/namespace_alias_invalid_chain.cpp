struct NoType {};
template <class T> struct Maybe { typedef T type; };
template <> struct Maybe<NoType> {};
template <class T> using TwoStep = typename Maybe<typename Maybe<T>::type>::type;

using Invalid = TwoStep<NoType>;
int main() { return 0; }
