# ADR 005: no exceptions or RTTI

Exceptions and RTTI are excluded from v1. Unsupported constructs produce diagnostics rather than silently
lowering to a different runtime model.
