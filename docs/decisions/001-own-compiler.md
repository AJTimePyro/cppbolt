# ADR 001: own compiler

The project implements its defined C++ subset directly instead of embedding LLVM. This keeps browser startup
and shipped size predictable and makes the accepted language explicit.
