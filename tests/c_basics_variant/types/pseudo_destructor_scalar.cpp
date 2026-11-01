#include <cstdio>

typedef int T;
typedef double D;

int g_index_evals = 0;
int get_index() {
    g_index_evals++;
    return 1;
}

int main() {
    T arr[3] = {10, 20, 30};
    arr[0].~T();
    arr[get_index()].~T();

    D darr[2] = {1.5, 2.5};
    darr[0].~D();

    std::printf("arr[0]=%d arr[1]=%d darr[0]=%.1f evals=%d\n", arr[0], arr[1], darr[0], g_index_evals);
    return 0;
}
