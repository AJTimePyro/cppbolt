// Test that unexpanded parameter pack in member declaration is rejected
template<typename... Ts> struct Bad { Ts member; };

int main() {
    return 0;
}
