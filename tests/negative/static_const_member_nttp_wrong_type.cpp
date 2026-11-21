struct C { static const double d = 7.0; };
template<int N> struct Box {};
Box<C::d> x;
