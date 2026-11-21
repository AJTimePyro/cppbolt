// A body error in the selected specialization is outside the immediate
// context and must never be treated as a substitution failure.
template <typename T> int bad(T x) { return x.no_such_member; }
int bad(...) { return 2; }

int main() {
    return bad(1);
}
