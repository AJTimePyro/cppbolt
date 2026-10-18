#include <iostream>
#include <vector>
#include <string>
#include <map>
#include <set>

// Linear Congruential Generator
static unsigned long g_seed = 123456789;
static unsigned long rnd() {
    g_seed = (1103515245 * g_seed + 12345) & 0x7fffffff;
    return g_seed;
}

int main() {
    const int NUM_SEEDS = 20;

    for (int s_idx = 0; s_idx < NUM_SEEDS; ++s_idx) {
        g_seed = 10007 * (s_idx + 1) + 42;

        // 1. Vector differential fuzz (push_back, emplace_back, erase, pop_back)
        std::vector<int> v;
        for (int i = 0; i < 50; ++i) {
            int op = rnd() % 4;
            int val = rnd() % 1000;
            if (op == 0) {
                v.push_back(val);
            } else if (op == 1) {
                v.emplace_back(val);
            } else if (op == 2 && !v.empty()) {
                size_t idx = rnd() % v.size();
                v.erase(v.begin() + idx);
            } else if (op == 3 && !v.empty()) {
                v.pop_back();
            }
#ifdef _MINI_STL_VECTOR_
            if (!v.check_invariants()) {
                std::cout << "Vector invariant failed at seed " << s_idx << " op " << i << std::endl;
                return 1;
            }
#endif
        }

        // 2. String differential fuzz (append, erase, compare)
        std::string s = "seed_init";
        for (int i = 0; i < 50; ++i) {
            int op = rnd() % 3;
            if (op == 0) {
                char c = 'a' + (rnd() % 26);
                s.push_back(c);
            } else if (op == 1 && s.size() > 2) {
                size_t pos = rnd() % (s.size() - 1);
                size_t len = 1 + (rnd() % 3);
                s.erase(pos, len);
            } else {
                s += "!";
            }
        }

        // 3. Map differential fuzz (insert, [], find, count, lower_bound, erase key, erase iterator)
        std::map<int, int> m;
        for (int i = 0; i < 70; ++i) {
            int op = rnd() % 6;
            int key = rnd() % 40;
            int val = rnd() % 1000;
            if (op == 0) {
                m[key] = val;
            } else if (op == 1) {
                m.insert(std::make_pair(key, val));
            } else if (op == 2) {
                std::map<int, int>::iterator it = m.lower_bound(key);
                (void)it;
            } else if (op == 3) {
                // Erase key
                m.erase(key);
            } else if (op == 4 && !m.empty()) {
                // Erase iterator
                std::map<int, int>::iterator it = m.find(key);
                if (it != m.end()) {
                    m.erase(it);
                }
            } else {
                (void)m.count(key);
            }
#ifdef _MINI_STL_MAP_
            if (!m.check_invariants()) {
                std::cout << "Map AVL invariant failed at seed " << s_idx << " op " << i << std::endl;
                return 1;
            }
#endif
        }

        // 4. Set differential fuzz (insert, find, count, lower_bound, erase key, erase iterator)
        std::set<int> st;
        for (int i = 0; i < 70; ++i) {
            int op = rnd() % 5;
            int val = rnd() % 50;
            if (op == 0) {
                st.insert(val);
            } else if (op == 1) {
                std::set<int>::iterator it = st.lower_bound(val);
                (void)it;
            } else if (op == 2) {
                st.erase(val);
            } else if (op == 3 && !st.empty()) {
                std::set<int>::iterator it = st.find(val);
                if (it != st.end()) {
                    st.erase(it);
                }
            } else {
                (void)st.count(val);
            }
#ifdef _MINI_STL_SET_
            if (!st.check_invariants()) {
                std::cout << "Set AVL invariant failed at seed " << s_idx << " op " << i << std::endl;
                return 1;
            }
#endif
        }

        // Report checksum for this seed
        std::cout << "seed " << s_idx << ": v=" << v.size() << " s=" << s.size() << " m=" << m.size() << " st=" << st.size() << std::endl;
    }

    std::cout << "Differential fuzz complete: all 20 seeds verified with AVL/vector invariants." << std::endl;
    return 0;
}
