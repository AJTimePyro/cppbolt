#include <cstdio>

void f(int& x) {
    std::printf("lvalue\n");
}

void f(const int& x) {
    std::printf("rvalue\n");
}

int g_val = 100;
int& get_ref() {
    return g_val;
}

int& g_ref = get_ref();

struct S {
    int m_val;
    S() : m_val(200) {}
    int& get_ref() { return m_val; }
};

int main() {
    int a = 10;
    f(a);
    f(42);
    f(get_ref());
    S s;
    f(s.get_ref());
    f(g_ref);
    return 0;
}
