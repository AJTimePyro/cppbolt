template <typename T> void call(T x) { g(x); }
void g(int) {}
int main() { call(1); }
