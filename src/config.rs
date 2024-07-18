pub const DEPS_ALWAYS_AVAIL: [&str; 23] = [
    "b3sum",
    "baselayout",
    "binutils",
    "bison",
    "busybox",
    "bzip2",
    "certs",
    "curl",
    "flex",
    "gcc",
    "git",
    "gmp",
    "kiss",
    "libmpc",
    "linux-headers",
    "m4",
    "make",
    "mpfr",
    "musl",
    "openssl",
    "pigz",
    "xz",
    "zlib",
];

pub const DEPS_MAKE: [&str; 7] = [
    "autoconf",
    "automake",
    "cmake",
    "meson",
    "nasm",
    "rust",
    "samurai",
];

pub const CMD_SEP: [char; 2] = [
    ';',
    '&',
];

pub const C_COMPILERS: [&str; 2] = [
    "gcc",
    "g++",
];
