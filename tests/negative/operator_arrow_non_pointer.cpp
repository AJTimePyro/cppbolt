struct BadProxy {
    int operator->() {
        return 42;
    }
};

int main() {
    BadProxy p;
    return p->val;
}
