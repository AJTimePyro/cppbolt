#ifndef _MINI_STL_DETAIL_AVL_TREE_
#define _MINI_STL_DETAIL_AVL_TREE_

#include <cstddef>
#include <new>

namespace std {
    namespace detail {
        template<typename T, typename Key, typename KeyOfValue, typename Compare>
        class AVLTree {
        public:
            struct Node {
                Node* parent;
                Node* left;
                Node* right;
                int height;
                T data;

                Node(const T& d, Node* p = nullptr) : parent(p), left(nullptr), right(nullptr), height(1), data(d) {}
            };

            struct iterator {
                Node* current;
                iterator(Node* n = nullptr) : current(n) {}

                T& operator*() const { return current->data; }
                T* operator->() const { return &current->data; }

                iterator& operator++() {
                    if (!current) return *this;
                    if (current->right) {
                        current = current->right;
                        while (current->left) current = current->left;
                    } else {
                        Node* p = current->parent;
                        while (p && current == p->right) {
                            current = p;
                            p = p->parent;
                        }
                        current = p;
                    }
                    return *this;
                }

                bool operator==(const iterator& o) const { return current == o.current; }
                bool operator!=(const iterator& o) const { return current != o.current; }
            };

            typedef iterator const_iterator;

        private:
            Node* root_;
            size_t size_;
            Compare comp_;
            KeyOfValue key_of_;

            int height(Node* n) const { return n ? n->height : 0; }
            int balance_factor(Node* n) const { return n ? height(n->left) - height(n->right) : 0; }
            void update_height(Node* n) {
                if (!n) return;
                int hl = height(n->left);
                int hr = height(n->right);
                n->height = 1 + (hl > hr ? hl : hr);
            }

            Node* rotate_right(Node* y) {
                Node* x = y->left;
                Node* T2 = x->right;

                x->right = y;
                y->left = T2;

                x->parent = y->parent;
                y->parent = x;
                if (T2) T2->parent = y;

                update_height(y);
                update_height(x);
                return x;
            }

            Node* rotate_left(Node* x) {
                Node* y = x->right;
                Node* T2 = y->left;

                y->left = x;
                x->right = T2;

                y->parent = x->parent;
                x->parent = y;
                if (T2) T2->parent = x;

                update_height(x);
                update_height(y);
                return y;
            }

            void destroy(Node* n) {
                if (!n) return;
                destroy(n->left);
                destroy(n->right);
                delete n;
            }

            bool check_node(Node* n, size_t& count) const {
                if (!n) return true;
                count++;
                if (n->left) {
                    if (n->left->parent != n) return false;
                    if (!comp_(key_of_(n->left->data), key_of_(n->data))) return false;
                }
                if (n->right) {
                    if (n->right->parent != n) return false;
                    if (!comp_(key_of_(n->data), key_of_(n->right->data))) return false;
                }
                int hl = height(n->left);
                int hr = height(n->right);
                if (n->height != 1 + (hl > hr ? hl : hr)) return false;
                int bf = hl - hr;
                if (bf < -1 || bf > 1) return false;
                return check_node(n->left, count) && check_node(n->right, count);
            }

        public:
            AVLTree() : root_(nullptr), size_(0) {}
            ~AVLTree() { destroy(root_); }

            size_t size() const { return size_; }
            bool empty() const { return size_ == 0; }

            iterator begin() const {
                if (!root_) return iterator(nullptr);
                Node* n = root_;
                while (n->left) n = n->left;
                return iterator(n);
            }

            iterator end() const { return iterator(nullptr); }

            Node* find_node(const Key& k) const {
                Node* cur = root_;
                while (cur) {
                    if (comp_(k, key_of_(cur->data))) {
                        cur = cur->left;
                    } else if (comp_(key_of_(cur->data), k)) {
                        cur = cur->right;
                    } else {
                        return cur;
                    }
                }
                return nullptr;
            }

            iterator find(const Key& k) const {
                return iterator(find_node(k));
            }

            size_t count(const Key& k) const {
                return find_node(k) ? 1 : 0;
            }

            iterator lower_bound(const Key& k) const {
                Node* cur = root_;
                Node* candidate = nullptr;
                while (cur) {
                    if (!comp_(key_of_(cur->data), k)) {
                        candidate = cur;
                        cur = cur->left;
                    } else {
                        cur = cur->right;
                    }
                }
                return iterator(candidate);
            }

            iterator upper_bound(const Key& k) const {
                Node* cur = root_;
                Node* candidate = nullptr;
                while (cur) {
                    if (comp_(k, key_of_(cur->data))) {
                        candidate = cur;
                        cur = cur->left;
                    } else {
                        cur = cur->right;
                    }
                }
                return iterator(candidate);
            }

            iterator erase(iterator pos) {
                if (!pos.current) return end();
                iterator next_it = pos;
                ++next_it;
                Key k = key_of_(pos.current->data);
                erase(k);
                return next_it;
            }

            size_t erase(const Key& k) {
                Node* z = find_node(k);
                if (!z) return 0;

                Node* node_to_rebalance = nullptr;

                if (!z->left || !z->right) {
                    Node* child = z->left ? z->left : z->right;
                    Node* p = z->parent;
                    if (child) child->parent = p;
                    if (!p) {
                        root_ = child;
                    } else if (p->left == z) {
                        p->left = child;
                    } else {
                        p->right = child;
                    }
                    node_to_rebalance = p;
                    delete z;
                } else {
                    Node* y = z->right;
                    while (y->left) y = y->left;

                    z->data = y->data;

                    Node* child = y->right;
                    Node* p = y->parent;
                    if (child) child->parent = p;
                    if (p->left == y) {
                        p->left = child;
                    } else {
                        p->right = child;
                    }
                    node_to_rebalance = p;
                    delete y;
                }
                size_--;

                Node* node = node_to_rebalance;
                while (node) {
                    update_height(node);
                    int bf = balance_factor(node);
                    Node* p = node->parent;
                    Node* new_subtree_root = node;

                    if (bf > 1) {
                        if (balance_factor(node->left) < 0) {
                            node->left = rotate_left(node->left);
                        }
                        new_subtree_root = rotate_right(node);
                    } else if (bf < -1) {
                        if (balance_factor(node->right) > 0) {
                            node->right = rotate_right(node->right);
                        }
                        new_subtree_root = rotate_left(node);
                    }

                    if (new_subtree_root != node) {
                        if (!p) {
                            root_ = new_subtree_root;
                            new_subtree_root->parent = nullptr;
                        } else if (p->left == node) {
                            p->left = new_subtree_root;
                            new_subtree_root->parent = p;
                        } else {
                            p->right = new_subtree_root;
                            new_subtree_root->parent = p;
                        }
                        node = new_subtree_root;
                    }

                    node = node->parent;
                }

                return 1;
            }

            Node* insert_internal(const T& val, bool& inserted) {
                inserted = false;
                if (!root_) {
                    root_ = new Node(val, nullptr);
                    size_++;
                    inserted = true;
                    return root_;
                }

                Node* cur = root_;
                Node* parent = nullptr;
                bool go_left = false;
                while (cur) {
                    parent = cur;
                    if (comp_(key_of_(val), key_of_(cur->data))) {
                        cur = cur->left;
                        go_left = true;
                    } else if (comp_(key_of_(cur->data), key_of_(val))) {
                        cur = cur->right;
                        go_left = false;
                    } else {
                        return cur;
                    }
                }

                Node* new_node = new Node(val, parent);
                if (go_left) {
                    parent->left = new_node;
                } else {
                    parent->right = new_node;
                }
                size_++;
                inserted = true;

                Node* node = parent;
                while (node) {
                    update_height(node);
                    int bf = balance_factor(node);
                    Node* p = node->parent;
                    Node* new_subtree_root = node;

                    if (bf > 1) {
                        if (balance_factor(node->left) < 0) {
                            node->left = rotate_left(node->left);
                        }
                        new_subtree_root = rotate_right(node);
                    } else if (bf < -1) {
                        if (balance_factor(node->right) > 0) {
                            node->right = rotate_right(node->right);
                        }
                        new_subtree_root = rotate_left(node);
                    }

                    if (new_subtree_root != node) {
                        if (!p) {
                            root_ = new_subtree_root;
                            new_subtree_root->parent = nullptr;
                        } else if (p->left == node) {
                            p->left = new_subtree_root;
                            new_subtree_root->parent = p;
                        } else {
                            p->right = new_subtree_root;
                            new_subtree_root->parent = p;
                        }
                        node = new_subtree_root;
                    }

                    node = node->parent;
                }

                return new_node;
            }

            bool check_invariants() const {
                size_t count = 0;
                if (!check_node(root_, count)) return false;
                if (count != size_) return false;
                return true;
            }
        };
    }
}

#endif
