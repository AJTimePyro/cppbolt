// A missing nested type in a default template argument rejects the candidate.
struct Bad {};
template <typename T, typename U = typename T::type> int f(T) { return 1; }
int main() { return f<Bad>(Bad{}); }
