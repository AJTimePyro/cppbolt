#include <cstdio>

int g_dtor_count = 0;

template <typename T>
struct Buffer {
    T data[2];

    Buffer() {
        data[0] = 0;
        data[1] = 0;
    }

    ~Buffer() {
        g_dtor_count++;
    }

    T& operator[](int i) {
        return data[i];
    }
};

void run_scope() {
    Buffer<int> buf;
    buf[0] = 7;
    buf[1] = 13;
    printf("%d %d\n", buf[0], buf[1]);
}

int main() {
    run_scope();
    printf("dtors=%d\n", g_dtor_count);
    return 0;
}
