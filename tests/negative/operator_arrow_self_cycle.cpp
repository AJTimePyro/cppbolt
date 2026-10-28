struct SelfLoop {
    SelfLoop operator->();
};

int main() {
    SelfLoop s;
    return s->val;
}
