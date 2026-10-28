struct Dummy {
    int val;
    Dummy(int v) : val(v) {}
};

int main() {
    Dummy d(1);
    // Arrow access on struct without operator-> should fail cleanly
    return d->val;
}
