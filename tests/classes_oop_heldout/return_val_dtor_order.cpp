#include <cstdio>

class Worker {
public:
    int slot;
    Worker(int s) : slot(s) {
        std::printf("Worker init %d\n", slot);
    }
    ~Worker() {
        std::printf("Worker deinit %d\n", slot);
    }
    int compute_metric() {
        std::printf("eval metric on slot %d\n", slot);
        return slot * 5;
    }
};

int run_pipeline() {
    Worker w1(3);
    Worker w2(7);
    return w1.compute_metric() + w2.compute_metric();
}

int main() {
    std::printf("start pipeline\n");
    int res = run_pipeline();
    std::printf("pipeline res: %d\n", res);
    return 0;
}
