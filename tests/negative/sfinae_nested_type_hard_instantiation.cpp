// Instantiating Holder<int> fails while resolving Holder<T>::type; that error is hard.
template <class T, class U> struct Ambig {};
template <class T> struct Ambig<T, int> {};
template <class T> struct Ambig<int, T> {};
template <class T> struct Holder {
    using type = int;
    Ambig<int, int> member;
};
template <class T> int f(typename Holder<T>::type* p) { return 1; }
template <class T> int f(T*) { return 2; }
int main() { return f<int>(nullptr); }
