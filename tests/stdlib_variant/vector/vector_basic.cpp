#include <iostream>
#include <vector>

int main() {
    std::vector<int> vi;
    vi.push_back(100);
    vi.push_back(200);
    vi.emplace_back(300);
    std::cout << "vi size: " << vi.size() << ", vi[1]: " << vi[1] << std::endl;

    std::vector<int> vi_copy = vi;
    vi_copy[1] = 999;
    std::cout << "orig vi[1]: " << vi[1] << ", copy vi[1]: " << vi_copy[1] << std::endl;

    vi.pop_back();
    std::cout << "vi size after pop: " << vi.size() << std::endl;
    vi.clear();
    std::cout << "vi empty after clear: " << (vi.empty() ? 1 : 0) << std::endl;

    std::vector<double> vd;
    vd.push_back(1.5);
    vd.push_back(2.5);
    std::cout << "vd size: " << vd.size() << ", vd[0]: " << vd[0] << ", vd[1]: " << vd[1] << std::endl;

    return 0;
}
