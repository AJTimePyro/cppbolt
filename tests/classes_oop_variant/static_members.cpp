#include <cstdio>

class SessionRegistry {
public:
    static int active_sessions;
    int session_id;

    SessionRegistry() {
        active_sessions++;
        session_id = active_sessions * 10;
    }
    ~SessionRegistry() {
        active_sessions--;
    }
    static int count() {
        return active_sessions;
    }
};

int SessionRegistry::active_sessions = 0;

int main() {
    std::printf("start: %d\n", SessionRegistry::count());
    SessionRegistry s1;
    std::printf("one: %d (id=%d)\n", SessionRegistry::count(), s1.session_id);
    {
        SessionRegistry s2;
        SessionRegistry s3;
        std::printf("nested: %d\n", SessionRegistry::count());
    }
    std::printf("end: %d\n", SessionRegistry::count());
    return 0;
}
