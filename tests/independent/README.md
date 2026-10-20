# Independent corpus

Tests in this directory are authored without access to the compiler source. An author may
use the C++17 specification, public compiler documentation, and the expected-output
procedure below, but must not read `src/`, the main corpus, or the variant corpus while
writing a batch.

After this directory was established, the implementing agent must never edit or delete
files in it. Only the repository owner may approve additions, corrections, or manifest
updates.

## Layout

Each test uses the same layout as the other positive groups:

- `<name>.cpp` is the source.
- `<name>.expected` is the required stdout.
- `<name>.input` is optional stdin.

Generate the expected output with `g++ -std=c++17 -O0`, then verify the same output with
`clang++ -std=c++17 -O0`. Both compilers must agree exactly.

## Expected failures

A test that passes must be listed in `../PASSING.txt`. A test that is absent from that
file is an expected failure: the runner still reports it, but it is not part of the
passing baseline. Record every expected failure below with one line in the form
`<name>: <reason>`.

No expected failures are recorded yet.

## Manifest

`MANIFEST.sha256` pins every file in this directory except itself. The runner fails when a
listed file changes or disappears, or when an unlisted file appears. Regenerate the
manifest only after the repository owner approves the file change:

```sh
cd tests/independent
find . -type f ! -name MANIFEST.sha256 -printf '%P\0' | sort -z | xargs -0 sha256sum > MANIFEST.sha256
```
