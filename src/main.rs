//! Rust consumer of oresoftware/flags-2-env.
//!
//! Asserts the contract in EXPECTED.md. Exits non-zero on the first
//! disagreement, which is what makes `docker run` the whole test.

use flags2env::Flags2Env;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

fn repo_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is baked in at compile time, so the binary finds its
    // config whatever directory it is run from.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

fn main() -> ExitCode {
    let root = repo_root();
    let config = root.join(".cli-flags.toml");
    let config = config.to_str().expect("config path is not valid UTF-8");

    // The crate compiles src/parser.c into itself through build.rs, but the
    // client's public API loads a shared object at runtime, so the fixture
    // exercises the same dlopen path the other FFI clients take.
    let library = env::var("FLAGS2ENV_NATIVE_LIB").unwrap_or_else(|_| {
        root.join(".vendor/.zed/oresoftware/flags-2-env/build/libflags2env.so")
            .to_string_lossy()
            .into_owned()
    });

    let sdk = match unsafe { Flags2Env::load(Some(&library)) } {
        Ok(sdk) => sdk,
        Err(error) => {
            eprintln!("could not load {library}: {error}");
            return ExitCode::FAILURE;
        }
    };

    let defaults = map(&[
        ("PORT", "3000"),
        ("DEBUG", "false"),
        ("APP_ENV", "development"),
        ("COLOR", "true"),
    ]);
    let overridden = map(&[
        ("PORT", "8181"),
        ("DEBUG", "true"),
        ("APP_ENV", "production"),
        ("COLOR", "true"),
    ]);
    let negated = map(&[
        ("PORT", "3000"),
        ("DEBUG", "false"),
        ("APP_ENV", "development"),
        ("COLOR", "false"),
    ]);

    let cases: Vec<(&str, Vec<&str>, &HashMap<String, String>)> = vec![
        ("defaults", vec![], &defaults),
        (
            "long flags",
            vec!["--port", "8181", "--debug=t", "--mode", "production"],
            &overridden,
        ),
        (
            "short flags",
            vec!["-p", "8181", "-d", "1", "--env", "production"],
            &overridden,
        ),
        (
            "long aliases",
            vec!["--listen-port", "8181", "--debug", "1", "--mode", "production"],
            &overridden,
        ),
        (
            "joined by =",
            vec!["--port=8181", "--debug=yes", "--mode=production"],
            &overridden,
        ),
        ("negation", vec!["--no-color"], &negated),
    ];

    let mut failures = 0usize;

    for (label, flags, expected) in &cases {
        let mut argv = vec!["demo".to_string()];
        argv.extend(flags.iter().map(|flag| (*flag).to_string()));

        let got = match sdk.parse(&argv, Some(config)) {
            Ok(got) => got,
            Err(error) => {
                eprintln!("FAIL {label}: parse returned an error: {error}");
                failures += 1;
                continue;
            }
        };

        let ok = got == **expected;
        if !ok {
            failures += 1;
        }
        println!(
            "{:<4} {:<13} demo {}",
            if ok { "ok" } else { "FAIL" },
            label,
            flags.join(" ")
        );

        let mut keys: Vec<&String> = expected.keys().collect();
        keys.sort();
        for key in keys {
            let value = got.get(key).map(String::as_str).unwrap_or("<missing>");
            println!("       {key}={value}");
        }
        if !ok {
            eprintln!("       expected {expected:?}");
            eprintln!("       got      {got:?}");
        }
    }

    if failures > 0 {
        eprintln!(
            "\nrust-app: {failures} of {} cases disagree with the contract",
            cases.len()
        );
        return ExitCode::FAILURE;
    }

    println!(
        "\nrust-app OK: {} cases, via libloading into oresoftware/flags-2-env",
        cases.len()
    );
    ExitCode::SUCCESS
}
