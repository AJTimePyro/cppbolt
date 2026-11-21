template <int N> struct Box { static const int value = N; };
int main() { return Box<undefined_constant>::value; }
