#include <cstdio>
#include <new>

struct Session {
    int token;
    Session(int token) : token(token) {
        printf("Session ctor %d\n", token);
    }
    ~Session() {
        printf("Session dtor %d\n", token);
    }
};

char mem[128];

void destroy(Session* s) { s->~Session(); }

int main() {
    Session* s = new (mem) Session(999);
    printf("Active session %d\n", s->token);
    destroy(s);
    printf("Session terminated\n");
    return 0;
}
