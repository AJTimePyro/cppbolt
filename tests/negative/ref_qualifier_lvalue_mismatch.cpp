struct OnlyRValue {
    int run() && { return 1; }
};

int main() {
    OnlyRValue obj;
    obj.run();
    return 0;
}
