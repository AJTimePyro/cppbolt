#include <cstdio>

class BaseService {
public:
    virtual void service_hook() {
        std::printf("Base service_hook\n");
    }
    BaseService() {
        std::printf("BaseService ctor -> invoke hook\n");
        service_hook();
    }
    virtual ~BaseService() {
        std::printf("BaseService dtor -> invoke hook\n");
        service_hook();
    }
};

class DatabaseService : public BaseService {
public:
    void service_hook() override {
        std::printf("DatabaseService hook\n");
    }
    DatabaseService() {
        std::printf("DatabaseService ctor -> invoke hook\n");
        service_hook();
    }
    ~DatabaseService() override {
        std::printf("DatabaseService dtor -> invoke hook\n");
        service_hook();
    }
};

int main() {
    std::printf("=== construct db service ===\n");
    DatabaseService db;
    std::printf("=== destruct db service ===\n");
    return 0;
}
