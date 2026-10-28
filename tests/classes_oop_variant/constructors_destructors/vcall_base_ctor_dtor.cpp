#include <cstdio>

void log(const char* message) { std::printf("%s\n", message); }

class BaseService {
public:
    virtual void service_hook() {
        std::printf("Base service_hook\n");
    }
    BaseService() {
        log("BaseService ctor -> invoke hook");
        service_hook();
    }
    virtual ~BaseService() {
        log("BaseService dtor -> invoke hook");
        service_hook();
    }
};

class DatabaseService : public BaseService {
public:
    void service_hook() override {
        std::printf("DatabaseService hook\n");
    }
    DatabaseService() {
        log("DatabaseService ctor -> invoke hook");
        service_hook();
    }
    ~DatabaseService() override {
        log("DatabaseService dtor -> invoke hook");
        service_hook();
    }
};

int main() {
    std::printf("=== construct db service ===\n");
    DatabaseService db;
    std::printf("=== destruct db service ===\n");
    return 0;
}
