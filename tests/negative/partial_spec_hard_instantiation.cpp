struct Bad { };
template <typename T> struct Traits { using type = typename T::type; };
template <typename T, typename U> struct Select { static const int v = 0; };
template <typename T> struct Select<T, decltype(alignof(typename Traits<T>::type))> {
    static const int v = 1;
};
int main() { Select<Bad, size_t> s; return s.v; }
