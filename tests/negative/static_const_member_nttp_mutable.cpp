struct C { static int n; };
template<int N> struct Box {};
Box<C::n> x;
