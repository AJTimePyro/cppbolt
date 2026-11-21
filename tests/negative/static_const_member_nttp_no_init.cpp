struct C { static const int n; };
template<int N> struct Box {};
Box<C::n> x;
