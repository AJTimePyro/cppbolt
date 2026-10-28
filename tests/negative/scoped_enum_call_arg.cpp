enum class HttpCode { OK = 200 };

void take_int(int x) {}

int main() {
    take_int(HttpCode::OK);
    return 0;
}
