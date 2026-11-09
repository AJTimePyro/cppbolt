struct Widget {
    void mutate() {}
};

int main() {
    const Widget w;
    w.mutate();
    return 0;
}
