//! Golden-output baseline for behavior-preserving refactors.
//!
//! `--golden <manifest>` writes `<feature>/<test> <wasm-sha256> <stdout-sha256>`
//! for every positive test that compiles. `--check-golden <manifest>` recomputes
//! the same values and reports added, removed, or changed entries.

use super::{FEATURES, Paths, cpp_files, input_path, run_wasm};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

pub(super) enum Mode {
    Write,
    Check,
}

pub(super) fn run(mode: Mode, manifest: &Path, compiler: PathBuf) -> ExitCode {
    let work = std::env::temp_dir().join(format!("cppwasm-golden-{}", std::process::id()));
    if let Err(error) = fs::create_dir_all(&work) {
        eprintln!("cannot create {}: {error}", work.display());
        return ExitCode::FAILURE;
    }
    let paths = Paths { compiler, work };

    let mut entries = BTreeMap::new();
    let mut skipped = 0usize;
    for feature in FEATURES {
        let directory = Path::new("tests").join(feature);
        if let Err(error) = fs::create_dir_all(paths.work.join(feature)) {
            eprintln!("cannot create work directory for {feature}: {error}");
            let _ = fs::remove_dir_all(&paths.work);
            return ExitCode::FAILURE;
        }
        let tests = match cpp_files(&directory) {
            Ok(tests) => tests,
            Err(error) => {
                eprintln!("cannot list {}: {error}", directory.display());
                let _ = fs::remove_dir_all(&paths.work);
                return ExitCode::FAILURE;
            }
        };
        for cpp in tests {
            let Some(name) = cpp.file_stem().and_then(OsStr::to_str) else {
                continue;
            };
            match hash_one(&paths, feature, name, &cpp) {
                Ok(Some(entry)) => {
                    entries.insert(format!("{feature}/{name}"), entry);
                }
                Ok(None) => skipped += 1,
                Err(error) => {
                    eprintln!("[FAIL] {feature}/{name}: {error}");
                    let _ = fs::remove_dir_all(&paths.work);
                    return ExitCode::FAILURE;
                }
            }
        }
    }
    let _ = fs::remove_dir_all(&paths.work);

    match mode {
        Mode::Write => write_manifest(manifest, &entries, skipped),
        Mode::Check => check_manifest(manifest, &entries),
    }
}

fn hash_one(
    paths: &Paths,
    feature: &str,
    name: &str,
    cpp: &Path,
) -> Result<Option<(String, String)>, String> {
    let wasm = paths.output(feature, name, "wasm");
    let compiled = Command::new("timeout")
        .arg("5s")
        .arg(&paths.compiler)
        .arg(cpp)
        .arg("-o")
        .arg(&wasm)
        .output()
        .map_err(|error| format!("cannot run compiler: {error}"))?;
    if !compiled.status.success() {
        return Ok(None);
    }

    let output = run_wasm(&wasm, &input_path(feature, name))
        .map_err(|error| format!("cannot run node: {error}"))?;
    let bytes =
        fs::read(&wasm).map_err(|error| format!("cannot read {}: {error}", wasm.display()))?;
    Ok(Some((sha256_hex(&bytes), sha256_hex(&output.stdout))))
}

fn write_manifest(
    manifest: &Path,
    entries: &BTreeMap<String, (String, String)>,
    skipped: usize,
) -> ExitCode {
    let mut text = String::new();
    for (name, (wasm, output)) in entries {
        text.push_str(name);
        text.push(' ');
        text.push_str(wasm);
        text.push(' ');
        text.push_str(output);
        text.push('\n');
    }
    if let Some(parent) = manifest.parent()
        && !parent.as_os_str().is_empty()
        && let Err(error) = fs::create_dir_all(parent)
    {
        eprintln!("cannot create {}: {error}", parent.display());
        return ExitCode::FAILURE;
    }
    if let Err(error) = fs::write(manifest, text) {
        eprintln!("cannot write {}: {error}", manifest.display());
        return ExitCode::FAILURE;
    }
    println!(
        "Golden baseline: {} entries written to {}, {} uncompilable tests skipped.",
        entries.len(),
        manifest.display(),
        skipped
    );
    ExitCode::SUCCESS
}

fn check_manifest(manifest: &Path, entries: &BTreeMap<String, (String, String)>) -> ExitCode {
    let text = match fs::read_to_string(manifest) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("cannot read {}: {error}", manifest.display());
            return ExitCode::FAILURE;
        }
    };
    let mut expected = BTreeMap::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let mut parts = line.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let Some(wasm) = parts.next() else {
            eprintln!("malformed golden entry: {line}");
            return ExitCode::FAILURE;
        };
        let Some(output) = parts.next() else {
            eprintln!("malformed golden entry: {line}");
            return ExitCode::FAILURE;
        };
        expected.insert(name.to_string(), (wasm.to_string(), output.to_string()));
    }

    let mut failures = 0usize;
    for (name, value) in &expected {
        match entries.get(name) {
            None => {
                eprintln!("[FAIL] golden test no longer compiles: {name}");
                failures += 1;
            }
            Some(actual) if actual != value => {
                eprintln!("[FAIL] golden hashes changed: {name}");
                failures += 1;
            }
            Some(_) => {}
        }
    }
    for name in entries.keys() {
        if !expected.contains_key(name) {
            eprintln!("[FAIL] golden test newly compiles: {name}");
            failures += 1;
        }
    }

    if failures == 0 {
        println!(
            "Golden baseline: all {} entries match {}.",
            entries.len(),
            manifest.display()
        );
        ExitCode::SUCCESS
    } else {
        eprintln!("{failures} golden entries differ");
        ExitCode::FAILURE
    }
}

/// A small, dependency-free SHA-256 implementation used for the golden
/// manifest. It is not part of the compiler and never sees untrusted input.
pub(super) fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let mut message = data.to_vec();
    let bit_len = (data.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in message.as_chunks::<64>().0 {
        sha256_compress(&mut state, chunk, &K);
    }

    state.iter().map(|word| format!("{word:08x}")).collect()
}

/// Mixes one 64-byte block into the SHA-256 state.
fn sha256_compress(state: &mut [u32; 8], chunk: &[u8], k: &[u32; 64]) {
    let mut words = [0u32; 64];
    for (index, word) in words.iter_mut().take(16).enumerate() {
        let start = index * 4;
        *word = u32::from_be_bytes([
            chunk[start],
            chunk[start + 1],
            chunk[start + 2],
            chunk[start + 3],
        ]);
    }
    for index in 16..64 {
        let s0 = words[index - 15].rotate_right(7)
            ^ words[index - 15].rotate_right(18)
            ^ (words[index - 15] >> 3);
        let s1 = words[index - 2].rotate_right(17)
            ^ words[index - 2].rotate_right(19)
            ^ (words[index - 2] >> 10);
        words[index] = words[index - 16]
            .wrapping_add(s0)
            .wrapping_add(words[index - 7])
            .wrapping_add(s1);
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];
    for (round, word) in k.iter().zip(words.iter()) {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ ((!e) & g);
        let temp1 = h
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(*round)
            .wrapping_add(*word);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);
        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;

    #[test]
    fn hashes_known_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
