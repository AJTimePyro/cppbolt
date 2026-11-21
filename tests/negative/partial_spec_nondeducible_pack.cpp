template<class...> struct Tuple {};

template<class... Ts>
struct S {};

template<class... Ts>
struct S<Tuple<typename Ts::type...>> {};

int main() { return 0; }
