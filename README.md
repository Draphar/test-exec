## test-exec - Test command line applications comfortably

[![travis-badge]][travis]
[![appveyor-badge]][appveyor]
[![crates.io-badge]][crates.io]
[![license-badge]][license]

[travis-badge]: https://img.shields.io/travis/com/Draphar/test-exec.svg?branch=master&style=flat-square
[travis]: https://travis-ci.com/Draphar/mkpasswd
[appveyor-badge]: https://img.shields.io/appveyor/ci/Draphar/test-exec.svg?style=flat-square
[appveyor]: https://ci.appveyor.com/project/Draphar/test-exec
[crates.io-badge]: https://img.shields.io/crates/v/test-exec.svg?style=flat-square
[crates.io]: https://crates.io/crates/test-exec
[license-badge]: https://img.shields.io/crates/l/test-exec.svg?style=flat-square
[license]: https://github.com/Draphar/test-exec/blob/master/LICENSE

*Cargo.toml*
```
[dev-dependencies]
test-exec = "0.1.0
```

`test-exec` is a Rust testing library to help you at testing the output of a command line application.
It aims for maximum comfort, and wants to prevent messing around with `Command`.

The main functionality is the [`exec`][exec-doc] macro:
it executes your command, verifies the output and is highly customizable.

A few previews, assuming you have a binary target called `my_bin`:

- minimum configuration:
    `exec!("my_bin");`
    
- (almost) maximum configuration:
```rust
let output = exec!{
    "my_bin",
    args: ["-p", "/"],
    cwd: "/tmp",
    env: {
        THREADS: "4"
    },
    stdin: b"show-hidden",
    timeout: 60000,
    log: true,

    code: 0,
    stdout: b"Started program...\nDone.\n",
    stderr: []
};

// `output` can be used here like a normal process::Output
```

If the program exits with any other code than `0`, a different `stdout` or `stderr`,
or is running longer than 60 seconds, a panic occurs.
    
As you might have noticed, the bin target is added to the PATH automatically.

See the [documentation][exec-doc] for more.

# Features

- set the arguments, current working directory, environment and `stdin` with one line each
- exit code, `stdout`, `stderr` and optionally termination signal comparison directly through the macro
- automatic availability of bin targets
- all output of the program is returned for additional use

# Installation and usage

As `test-exec` is a testing library, it should be added to the dev-dependencies:

```toml
[dev-dependencies]
test-exec = "0.1.0
```

And it can be used in code by doing

```rust
#[macro_use]
extern crate test_exec;
```

For instance in an integration test for a binary called `my_pwd`, whichs prints the current working directory

*tests/bin.rs*
```rust
#[macro_use]
extern crate test_exec;

#[test]
fn test_program_output() {
    exec!{
        "my_pwd",
        cwd: "/",
        log: true,
        
        code: 0,
        stdout: b"/\n",
        stderr: []
    };
}
```

[exec-doc]: https://docs.rs/test-exec/0.1.0/test-exec/exec
