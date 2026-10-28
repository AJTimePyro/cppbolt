#include <cstdio>

enum State {
    IDLE = 10,
    RUNNING = 20,
    PAUSED = 25,
    STOPPED = 30
};

const char* describe(State s) {
    switch (s) {
        case IDLE: return "idle";
        case RUNNING: return "running";
        case PAUSED: return "paused";
        case STOPPED: return "stopped";
        default: return "unknown";
    }
}

int main() {
    std::printf("%s\n", describe(IDLE));
    std::printf("%s\n", describe(RUNNING));
    std::printf("%s\n", describe(PAUSED));
    std::printf("%s\n", describe(STOPPED));
    return 0;
}
