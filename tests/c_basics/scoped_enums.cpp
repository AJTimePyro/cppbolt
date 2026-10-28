#include <cstdio>

enum class HttpCode {
    OK = 200,
    NOT_FOUND = 404,
    SERVER_ERROR = 500
};

enum class AlertLevel : int {
    INFO,
    WARNING,
    CRITICAL
};

int main() {
    HttpCode code = HttpCode::NOT_FOUND;
    switch (code) {
        case HttpCode::OK: std::printf("ok\n"); break;
        case HttpCode::NOT_FOUND: std::printf("not found: %d\n", (int)code); break;
        case HttpCode::SERVER_ERROR: std::printf("error\n"); break;
    }
    AlertLevel lvl = AlertLevel::CRITICAL;
    std::printf("level=%d\n", (int)lvl);
    return 0;
}
