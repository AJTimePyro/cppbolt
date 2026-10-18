#include <cstdio>

int main() {
    const char* msg = "Testing heldout pointer assignment preservation across multiple pointers.";
    const char* ptr1 = 0;
    const char* ptr2 = 0;
    ptr1 = msg;
    ptr2 = ptr1;
    std::printf("%s\n", ptr2);

    char buf[8];
    buf[0] = 'Y';
    buf[1] = 'E';
    buf[2] = 'S';
    buf[3] = '\0';
    char* pbuf = 0;
    pbuf = buf;
    std::printf("%s\n", pbuf);
    return 0;
}
