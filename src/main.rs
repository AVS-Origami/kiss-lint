use std::error::Error;
use std::fs;
use std::process::{self, Command};

mod config;

type Res<T> = Result<T, Box<dyn Error>>;

fn main() -> Res<()> {
    let kiss_c = Command::new("kiss").arg("c").status()?;
    if ! kiss_c.success() {
        die("kiss c failed. Fix issues then try again.");
    }

    let shellcheck = Command::new("shellcheck").arg("build").status()?;
    if ! shellcheck.success() {
        die("shellcheck failed. Fix issues then try again.");
    }

    let mut rep = Reporter::new();
    
    let version = fs::read_to_string("version")?;
    let version_fields: Vec<&str> = version.split_whitespace().collect();
    match version_fields.len() {
        2 => if version_fields[0] == "9999" { eprintln!("version: use git instead of 9999 (#1602)") },
        _ => eprintln!("version: too many fields; expected upstream and relative version number (#1603)"),
    }

    if fs::metadata("sources").is_ok() {
        let sources = fs::read_to_string("sources")?;
        rep.file("sources");
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

    if fs::metadata("depends").is_ok() {
        let depends = fs::read_to_string("depends")?;
        rep.file("depends");
        rep.i = 0;
        
        if depends.trim().len() == 0 {
            rep.err("empty depends file (#1206)");
        } else {
            for (i, line) in depends.lines().enumerate() {
                rep.i = i;

                for dep in &config::DEPS_ALWAYS_AVAIL {
                    if line.starts_with(dep) {
                        rep.err(&format!("dependency {dep} is always available (#1202)"));
                    }
                }

                for dep in &config::DEPS_MAKE {
                    if line.starts_with(dep) &&! line.ends_with(" make") {
                        rep.err(&format!("build dependency {dep} is listed as runtime (#1203)"));
                    }
                }
            }

            let lines: Vec<&str> = depends.lines().collect();
            let mut lines_sorted: Vec<&str> = lines.clone();
            lines_sorted.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

            if lines_sorted != lines {
                rep.err("depends file not sorted (#1205)");
            }
        }
    }

    let build = fs::read_to_string("build")?;
    rep.file("build");
    let mut indent = vec![];
    for (i, line) in build.lines().enumerate() {
        rep.i = i;

        let mut line_started = false;
        for (j, c) in line.chars().enumerate() {
            if c != ' ' &&! line_started {
                line_started = true;
                match j % 4 {
                    0 => {
                        if c.is_whitespace() {
                            rep.err("incorrect indentation; use four spaces (#0202)");
                        } else {
                            indent.push(j);
                        }

                        if indent.len() >= 2 && j != 0 {
                            let last_idx = indent.len() - 1;
                            let ident_diff = (indent[last_idx] as i64 - indent[last_idx - 1] as i64).abs() as usize;
                            match ident_diff {
                                0 | 4 => (),
                                x => {
                                    rep.err("incorrect indentation: use four spaces (#0202)");
                                    indent[last_idx] = j - x;
                                },
                            }
                        }
                    },
                    _ => {
                        rep.err("incorrect indentation; use four spaces (#0202)");
                        indent.push(0);
                    },
                }
            }
        }

        if line.len() > 80 {
            rep.err("line exceeds 80 chars (#0203)");
        }

        if i == 0 &&! line.starts_with("#!/bin/sh -e") {
            rep.err("missing or incorrect POSIX shebang (#0204)");
        }

        if i == 1 && line != "" {
            rep.err("missing newline after shebang (#0204)");
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        for tok in tokens {
            if tok.contains("\"$") &&! (tok.starts_with("\"") && tok.ends_with("\"")) {
                rep.err("quote entire string instead of variable (#0209)");
            }
        }

        let cmds: Vec<&str> = line.split(&config::CMD_SEP).collect();
        for c in cmds {
            let c = c.trim();
            for comp in &config::C_COMPILERS {
                if c.starts_with(comp) {
                    rep.err(&format!("use $CC instead of {comp} (#0212)"));
                }
            }

            if c.starts_with("mkdir") {
                let c_parts: Vec<&str> = c.split_whitespace().collect();
                if c_parts.len() > 1 {
                    if !(c_parts[1].starts_with("-") && c_parts[1].contains("p")) {
                        rep.err("use mkdir with -p flag (#0213)");
                    }
                } else {
                    rep.err("use mkdir with -p flag (#0213)");
                }
            }

            if c.starts_with("echo") {
                rep.err("use printf instead of echo (#0214)");
            }
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
    pub tmp_ok: bool,
    pub i: usize,
    pub file: &'static str,
}

impl Reporter {
    pub fn new() -> Self {
        Reporter {
            ok: true,
            tmp_ok: true,
            i: 0,
            file: "",
        }
    }

    pub fn file(&mut self, file: &'static str) {
        self.file = file;
        if !self.tmp_ok { eprintln!(); }
        self.tmp_ok = true;
    }

    pub fn err(&mut self, msg: &str) {
        self.ok = false;
        self.tmp_ok = false;
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
