void recurse(int x) {
    recurse(x + 1);
}

int main() {
    recurse(1);
    return 0;
}
