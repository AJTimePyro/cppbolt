// A failure while instantiating Boom<Tag> is outside the immediate context of
// the caller's substitution, so it stays a hard error even with a fallback.
struct Tag {};
template <class T> struct Boom { typedef typename T::missing type; };
template <class T> typename Boom<T>::type f(T, int) { return 1; }
long f(Tag, double) { return -1; }
int main() { return (int)f(Tag(), 0); }
