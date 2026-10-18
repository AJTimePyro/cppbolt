#include <cstdio>
#include <cstdlib>

struct Node {
    int key;
    int val;
    int height;
    Node* left;
    Node* right;
};

int get_height(Node* n) { return n ? n->height : 0; }
int max(int a, int b) { return a > b ? a : b; }

Node* rotate_right(Node* y) {
    Node* x = y->left;
    Node* T2 = x->right;
    x->right = y;
    y->left = T2;
    y->height = max(get_height(y->left), get_height(y->right)) + 1;
    x->height = max(get_height(x->left), get_height(x->right)) + 1;
    return x;
}

Node* rotate_left(Node* x) {
    Node* y = x->right;
    Node* T2 = y->left;
    y->left = x;
    x->right = T2;
    x->height = max(get_height(x->left), get_height(x->right)) + 1;
    y->height = max(get_height(y->left), get_height(y->right)) + 1;
    return y;
}

int get_balance(Node* n) { return n ? get_height(n->left) - get_height(n->right) : 0; }

Node* insert(Node* node, int key, int val) {
    if (!node) {
        Node* n = (Node*)malloc(sizeof(Node));
        n->key = key;
        n->val = val;
        n->height = 1;
        n->left = 0;
        n->right = 0;
        return n;
    }
    if (key < node->key) node->left = insert(node->left, key, val);
    else if (key > node->key) node->right = insert(node->right, key, val);
    else { node->val = val; return node; }

    node->height = 1 + max(get_height(node->left), get_height(node->right));
    int balance = get_balance(node);

    if (balance > 1 && key < node->left->key) return rotate_right(node);
    if (balance < -1 && key > node->right->key) return rotate_left(node);
    if (balance > 1 && key > node->left->key) {
        node->left = rotate_left(node->left);
        return rotate_right(node);
    }
    if (balance < -1 && key < node->right->key) {
        node->right = rotate_right(node->right);
        return rotate_left(node);
    }
    return node;
}

Node* find(Node* root, int key) {
    Node* curr = root;
    while (curr) {
        if (key == curr->key) return curr;
        if (key < curr->key) curr = curr->left;
        else curr = curr->right;
    }
    return 0;
}

void free_tree(Node* n) {
    if (!n) return;
    free_tree(n->left);
    free_tree(n->right);
    free(n);
}

unsigned int rng_m = 987654321;
unsigned int next_rand_m() {
    rng_m ^= rng_m << 13;
    rng_m ^= rng_m >> 17;
    rng_m ^= rng_m << 5;
    return rng_m;
}

int main() {
    const int N = 200000;
    Node* root = 0;
    for (int i = 0; i < N; ++i) {
        int k = (int)(next_rand_m() % 1000000);
        root = insert(root, k, i);
    }
    int found = 0;
    for (int i = 0; i < N; ++i) {
        int k = (int)(next_rand_m() % 1000000);
        if (find(root, k)) found++;
    }
    printf("AVL map %d ops: height=%d, lookups found=%d\n", N, get_height(root), found);
    free_tree(root);
    return 0;
}
