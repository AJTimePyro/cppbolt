template <typename T>
void func(T val) {
    int s = sizeof...(val);
}

int main() {
    func(42);
    return 0;
}
