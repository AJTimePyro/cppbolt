# ADR 002: direct Wasm emission

The backend writes Wasm binary sections directly. Generated-code optimization is out of scope; the benefit is
a small dependency-free compiler and a transparent target ABI.
