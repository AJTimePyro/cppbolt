// The ambiguous partial specialization is instantiated while resolving the
// signature type. That error comes from instantiating another template and
// must stay a hard error.
template <class T, class U> struct Ambig {};
template <class T> struct Ambig<T, int> {};
template <class T> struct Ambig<int, T> {};

template <class T> int f(Ambig<int, int>* p, T) { return 1; }
template <class T> int f(T, T) { return 2; }

int main() {
    int x = 0;
    return f((Ambig<int, int>*)0, x);
}
