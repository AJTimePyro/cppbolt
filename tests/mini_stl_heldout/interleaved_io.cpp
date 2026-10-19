#include <iostream>
#include <cstdio>

int main() {
    int a = 0;
    std::cin >> a;

    int c1 = std::getchar();
    char c2 = 0;
    std::cin.get(c2);

    int b = 0;
    std::scanf("%d", &b);

    char s[32] = {0};
    std::scanf("%s", s);

    long d = 0;
    std::cin >> d;

    int peek_ch = std::cin.peek();

    char c3 = 0;
    std::cin >> c3;

    const char* note = "plain";
    if (peek_ch == ' ') {
        note = "spaced";
    }

    std::cout << "a=" << a << " c1=" << c1 << " c2=" << c2
              << " b=" << b << " s=" << s << " d=" << d
              << " note=" << note << " c3=" << c3 << "\n";
    return 0;
}
