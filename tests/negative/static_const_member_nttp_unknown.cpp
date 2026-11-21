struct C { static const int n = unknown_value; };
template<int N> struct Box {};
Box<C::n> x;
