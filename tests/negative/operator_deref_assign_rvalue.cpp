struct ValueWrapper {
    int val;
    ValueWrapper(int v) : val(v) {}
    int operator*() { return val; }
};

int main() {
    ValueWrapper w(10);
    *w = 20;
    return 0;
}
