#include <cstdio>

class LocalRAII {
public:
    int id;
    LocalRAII(int id_) : id(id_) {
        std::printf("LocalRAII ctor %d\n", id);
    }
    ~LocalRAII() {
        std::printf("LocalRAII dtor %d\n", id);
    }
    int get_val() {
        std::printf("evaluating return expr for id=%d\n", id);
        return id * 10;
    }
};

int compute() {
    LocalRAII a(1);
    LocalRAII b(2);
    return a.get_val() + b.get_val();
}

int main() {
    std::printf("before compute\n");
    int res = compute();
    std::printf("after compute: res=%d\n", res);
    return 0;
}
