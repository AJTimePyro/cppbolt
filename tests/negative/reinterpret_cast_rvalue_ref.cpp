int main() {
    int& r = reinterpret_cast<int&>(42);
    return 0;
}
