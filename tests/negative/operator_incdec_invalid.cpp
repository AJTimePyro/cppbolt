struct Dummy {
    int val;
    Dummy(int v) : val(v) {}
};

int main() {
    Dummy d(1);
    d++;
    return d.val;
}
