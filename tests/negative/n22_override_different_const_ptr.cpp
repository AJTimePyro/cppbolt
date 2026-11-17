struct B { virtual int f(char* p) { return 1; } };
struct D : B { int f(const char* p) override { return 2; } };
int main() { return 0; }
