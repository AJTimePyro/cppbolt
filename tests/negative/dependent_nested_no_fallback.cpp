struct NoType {};
template <class T> struct Maybe { typedef T type; };
template <> struct Maybe<NoType> {};

template <class T> typename Maybe<T>::type only(T t) { return t; }
int main() { return only(NoType()); }
