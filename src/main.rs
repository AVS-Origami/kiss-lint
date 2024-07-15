use std::error::Error;
use std::fs;
use std::process::{self, Command};

type Res<T> = Result<T, Box<dyn Error>>;

fn main() -> Res<()> {
    let kiss_c = Command::new("kiss").arg("c").status()?;
    if ! kiss_c.success() {
        die("\"kiss c\" failed. Fix issues then try again.");
    }

    let shellcheck = Command::new("shellcheck").arg("build").status()?;
    if ! shellcheck.success() {
        die("\"shellcheck\" failed. Fix issues then try again.");
    }

    let mut rep = Reporter::new();
    
    let version = fs::read_to_string("version")?;
    let version_fields: Vec<&str> = version.split_whitespace().collect();
    match version_fields.len() {
        2 => if version_fields[0] == "9999" { eprintln!("version: use 'git' instead of 9999 (#1602)") },
        _ => eprintln!("version: too many fields; expected upstream and relative version number (#1603)"),
    }

    if fs::metadata("sources").is_ok() {
        let sources = fs::read_to_string("sources")?;
        rep.file = "sources";
        for (i, line) in sources.lines().enumerate() {
            rep.i = i;

            if ! (line.starts_with("https") || line.starts_with("git+https")) {
                rep.err("found non-https source (#1401)");
            }

            if (line.starts_with("https") || line.starts_with("http")) && line.ends_with(".patch") {
                rep.err("patches should not be remote (#1402)");
            }

            if line.starts_with("git+") {
                rep.err("found git source; prefer release tarball if available (#1403)");
            }

            if line.contains("://www.") {
                rep.err("found www. (#1404)");
            }

            if line.ends_with(".git") {
                rep.err("found .git (#1404)");
            }
        }
    } else {
        warn("no sources file found.");
    }

    let build = fs::read_to_string("build")?;
    rep.file = "build";
    for (i, line) in build.lines().enumerate() {
        rep.i = i;

        if i == 0 &&! line.starts_with("#!/bin/sh -e") {
            rep.err("missing or incorrect POSIX shebang (#0204)");
        }

        if i == 1 && line != "" {
            rep.err("missing newline after shebang (#0204)");
        }

        let mut line_started = false;
        for (i, c) in line.chars().enumerate() {
            if c != ' ' &&! line_started {
                line_started = true;
                match i % 4 {
                    0 => (),
                    _ => rep.err("incorrect indentation; use four spaces (#0202)"),
                }
            }
        }

        if line.len() > 80 {
            rep.err("line exceeds 80 chars (#0203)");
        }
    }


    if rep.ok {
        eprintln!("All checks passed.");
    } else {
        eprintln!("Some issues found. See https://kisscommunity.bvnf.space/kiss/style-guide for more information.");
    }

    Ok(())
}

struct Reporter {
    pub ok: bool,
    pub i: usize,
    pub file: &'static str,
}

impl Reporter {
    pub fn new() -> Self {
        Reporter {
            ok: true,
            i: 0,
            file: "",
        }
    }

    pub fn err(&mut self, msg: &str) {
        self.ok = false;
        eprintln!("{} @ line {}: {msg}", self.file, self.i + 1);
    }
}

fn warn(msg: &str) {
    eprintln!("\x1b[33mWARNING\x1b[0m {msg}");
}

fn die(msg: &str) {
    eprintln!("\x1b[31mERROR\x1b[0m {msg}");
    process::exit(1);
}
