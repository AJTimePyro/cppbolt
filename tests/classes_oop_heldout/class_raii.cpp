#include <cstdio>

class BufferLock {
    int tag;
    int depth;
public:
    BufferLock(int t, int d) : tag(t), depth(d) {
        std::printf("Acquire lock [%d] at depth %d\n", tag, depth);
    }
    ~BufferLock() {
        std::printf("Release lock [%d] at depth %d\n", tag, depth);
    }
    void flush() {
        std::printf("Flush buffer [%d]\n", tag);
    }
};

void run_subtask() {
    BufferLock sub(404, 2);
    sub.flush();
}

int main() {
    std::printf("Session start\n");
    {
        BufferLock main_lock(200, 1);
        main_lock.flush();
        run_subtask();
    }
    std::printf("Session end\n");
    return 0;
}
