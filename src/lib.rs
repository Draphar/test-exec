/*!
## test-exec - Test your command line applications comfortably

This crate provides the [`exec!`] macro for testing binary targets.
It executes your command, verifies the output and is highly customizable.

A few previews, assuming you have a binary target called `my_bin`:

- minimum configuration:
    `exec!("my_bin");`

- (almost) maximum configuration:
```
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

## Input

The program name is the only required parameter and does not need a key.

Arguments can be set using the `args` key, which accepts iterable objects.
Just when using the [`Command::args()`] function must different arguments be splitted.

The current working directory can be set by the `cwd` key.

The environment can be modified using the `env` key,
which is a pseudo-object of environment variables.
To clear the environment, the `clear_env` key can be set to `false`.

The program's `stdin` is set by the `stdin` key.

A maximum running time can be configured through the `timeout` key.
Programs run indefinitely by default.

The output can be logged to `stdout` for debug purposes by setting `log` to `true`.

## Comparing the output

The `code` key is used to assert the program's exit code. It is always compared.

The `stdout` and `stderr` keys are used to assert the program's standard streams.

On Unix, the `signal` key is used to assert the signal that terminated the program.

If further inspection is required, the macro returns an [`Output`] struct,
which exposes an `ExitStatus` and two `Vec`s for the streams.

## Auto PATH modification

If the `modify_key` is set, the local PATH variable is altered and the crate's respective `target/release` and `target/debug`
are added to the front of the PATH, to make them have maximum priority. The release binarys have more priority.

If a custom PATH path is provided via the `env` key, it is modified as well.

[`exec!`]: macro.exec.html
[`Command::args()`]: https://doc.rust-lang.org/stable/std/process/struct.Command.html#method.args
[`Output`]: https://doc.rust-lang.org/stable/std/process/struct.Output.html
*/

#![allow(unused)]

extern crate wait_timeout;

use std::ffi::{OsString, OsStr};
use std::collections::VecDeque;
use std::env::{split_paths, join_paths};
use std::path::PathBuf;
use wait_timeout::ChildExt;
use std::process::{Child, ExitStatus};
use std::time::Duration;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::fmt::Debug;

/// Comfortably execute a command and its output.
///
/// This macro supports a variety of input.
///
/// # Input configuration
///
/// ## `args` (`impl IntoIter<Item = impl AsRef<OsStr>>`)
///
/// Configure the arguments to pass to the executable.
/// Array literals and `vec![]` declarations are possible.
///
/// ```
/// exec!{
///     "echo",
///     args: ["Hello world!"]
/// };
/// ```
///
/// ## `cwd` (`impl AsRef<OsStr>`)
///
/// Set the current working directory of the program.
///
/// ```
/// #[cfg(unix)]
/// exec!{
///     "pwd",
///     cwd: "/tmp"
/// };
///
/// #[cfg(windows)]
/// exec!{
///     "cd",
///     cwd: "/system32"
/// };
/// ```
///
/// ## `env` and `clear_env` (`bool`)
///
/// Modify enviroment variables. `env` is a pseudo-object with
/// raw identifiers as keys and expressions where `impl AsRef<OsStr>`
/// as values, separated by commas.
///
/// The `clear_env` option clears all environment variables *before*
/// applying the `env` values. It is set to `false` by default.
/// Remember to keep the path of the program in the path when using this key.
///
/// ```
/// exec!{
///     "printenv",
///     clear_env: true,
///     env: {
///         USER: "root",
///         COFFEE: "a lot"
///     }
/// };
/// ```
///
/// ## `modify_path` (`bool`)
///
/// By default, this is set to `true`, so the `target/debug` and `target/release` directorys are added to the PATH automatically.
/// This modification also happens if a custom PATH is provided via the `env` key.
///
/// If this behavior is undesired, this key can be set to `false`.
///
/// ```
/// exec!{
///     "printenv",
///     args: ["PATH"],
///     env: {
///         PATH: "/bin"
///     },
///     modify_path: false,
///
///     stdout: b"/bin\n"
/// };
/// ```
///
/// ## stdin (`impl AsRef<[u8]>`)
///
/// Set data to write to the program's stdin.
///
/// ```
/// exec!{
///     "cat",
///     stdin: b"meow"
/// };
/// ```
///
/// ## `timeout` (`u64`)
///
/// Set the maximum running time for the program in *milliseconds*.
/// When the timeout is exceeded, the process is killed using `SIGKILL` on Unix.
/// The exit code is set to the signal number, which is `9` on Unix.
/// This exit code is often unexpected, so use this carefully and only when
/// the test if supposed to fail after a certain time.
///
/// The maximum time depends on the operating system,
/// being `i32::max_value()` on Unix and `u32::max_value()` on Windows
/// (no warranty).
///
/// If `timeout` is not given, the program will never timeout.
///
/// ```
/// exec!{
///     "sleep",
///     args: ["60"],
///     timeout: 5000 // milliseconds!
/// };
/// ```
///
/// ## `log` (`bool`)
///
/// By setting this to `true`, the output of the program is logged after
/// successful execution.
///
/// Remember to pass `--nocapture` to tests using this option.
///
/// # Output comparison
///
/// `exec` offers various ways to compare the output of the program directly through the macro.
/// But, for dynamic output, the [`Output`] struct is returned.
///
/// ```
///	let output = exec!("pwd");
/// assert_eq!(output, b"/");
/// ```
///
/// [`Output`]: https://doc.rust-lang.org/stable/std/process/struct.Output.html
///
/// ## `code` (`i32`)
///
/// Make sure the exit code equals this code, panic if otherwise.
/// This property is always compared, and the default is `0`.
///
/// ```
/// exec!{
///     "false",
///     code: 1
/// };
/// exec!{
///     "true",
///     code: 0 // unnecessary, but may be used for more explicitness
/// };
/// ```
///
/// ## `stdout` and `stderr` (`impl AsRef<[u8]>`)
///
/// Make sure the program's `stdout` and `stderr` are exactly these bytes.
/// This is not compared if omitted.
/// Keep in my mind that many programs append a `\n` to their stdout.
///
/// ```
/// exec!{
///     "echo",
///     args: ["Hello", "world"],
///
///     stdout: b"Hello world\n",
///     stderr: []
/// };
/// ```
///
/// ## `signal` (`i32`)
///
/// > ! Supported Unix only, ignored on Windows
///
/// Make sure the signal that terminated the program equals this signal.
/// It is required that the program was definitely stopped by this signal,
/// otherwise the assertion will fail.
///
/// ```
/// exec!{
///     "sleep",
///     args: ["60"],
///     timeout: 1000,
///
///     code: 9,
///     signal: 9 // SIGKILL = 9
/// };
/// ```
#[macro_export]
macro_rules! exec {
    (
        // Input Configuration
        $program:expr // the program name, without a key and required
        $(, args: $args:expr)? // the arguments to pass to the program
        $(, cwd: $cwd:expr)? // the desired current working directory
        $(, clear_env: $clear_env:expr)? // a boolean setting whether the environment should be cleared
        $(, env: { $( $key:ident : $value:expr ),* } )? // the environment variables as pseudo-object
        $(, modify_path: $modify_path:expr)? // enable the auto-modification of the PATH to include bin targets
        $(, stdin: $stdin:expr)? // what to write to the program's stdin
        $(, timeout: $timeout:expr)? // maximum allowed running time of the program
        $(, log: $log:expr)? // log the output

        // Assertions
        $(, code: $code:expr)? // the expected exit code
        $(, stdout: $stdout:expr)? // the expected stdout
        $(, stderr: $stderr:expr)? // the expected stderr
        $(, signal: $signal:expr)? // expected signal ID of the signal that terminated the program
    ) => {{
        use $crate::*;
        use std::process::{Command, Stdio, Output};
        use std::env::var_os;
        use std::io::{self, Write};
        use std::time::Duration;
        #[cfg(unix)]
        use std::os::unix::process::ExitStatusExt;
        use std::mem;
        use std::str;
        use std::fmt::Debug;

        let mut command = Command::new($program);

        command.stdin(Stdio::null());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // $args
        $(
            command.args($args.into_iter());
        )?

        $( // $cwd
            command.current_dir($cwd);
        )?

        $( // $clear_env
            if $clear_env {
                command.env_clear();
            };
        )?

        let mut modify_path = true;
        let mut custom_path = false;
        $(
            modify_path = $modify_path;
        )?

        $( // $key, $value
            $(
                let key = stringify!($key);
                if key == "PATH" && modify_path {
                    command.env("PATH", alter_path($value, env!("CARGO_MANIFEST_DIR")));
                    custom_path = true;
                }else {
                     command.env(key, $value);
                };
            )*
        )?

        if !custom_path && modify_path {
            let path = alter_path(&var_os("PATH").unwrap_or_default(), env!("CARGO_MANIFEST_DIR"));
            command.env("PATH", path);
        };

        $( // $stdin
            let _ = $stdin; // trigger this only when $stdin is present
            command.stdin(Stdio::piped());
        )?

        let mut child = command.spawn().expect("Failed to spawn child process");
        $( // $stdin
            let stdin = &$stdin;
            let a = AsRef::<[u8]>::as_ref(&stdin);  // this syntax gives pretty, unambiguous type errors
            child.stdin.as_mut()
                .map(|buf|  buf.write_all(a).expect("Failed to write to stdin"));
        )?

        let mut duration = None;
        $( // $timeout
            duration = Some(Duration::from_millis($timeout as u64));
        )?
        let status = wait(&mut child, duration);
        let mut code = 0;
        $( // $code
            code = $code;
        )?
        assert(&code, &get_code(status), "Unexpected exit code");

        $( // $signal
            #[cfg(unix)] {
                assert_eq!(Some($signal), status.signal(), "Unexpected signal");
            };
        )?

        let mut stdout = Vec::with_capacity(0xff);
        let mut child_stdout = mem::replace(&mut child.stdout, None).unwrap();
        io::copy(&mut child_stdout, &mut stdout).unwrap();

        let mut stderr = Vec::with_capacity(0xf);
        let mut child_stderr = mem::replace(&mut child.stderr, None).unwrap();
        io::copy(&mut child_stderr, &mut stderr).unwrap();

        $( // $log
            if $log {
                println!("{} returned\n   code: {}\n   stdout: {:?}\n   stderr: {:?}",
                    stringify!($program),
                    code,
                    match str::from_utf8(&stdout) {
                        Ok(ref string) => string as &Debug,
                        Err(_) => &stdout
                    },
                    match str::from_utf8(&stderr) {
                        Ok(ref string) => string as &Debug,
                        Err(_) => &stderr
                    }
                );
            };
        )?

        $( // $stdout
            let expected_stdout = &$stdout;
            let a = AsRef::<[u8]>::as_ref(expected_stdout);
            assert(&a, &stdout.as_slice(), "Unexpected value of stdout");
        )?
        $( // $stderr
            let expected_stderr = &$stderr;
            let a = AsRef::<[u8]>::as_ref(expected_stderr);
            assert(&a, &stderr.as_slice(), "Unexpected value of stderr");
        )?

        let output = Output {
            status,
            stdout,
            stderr
        };

        output
    }};
    ($program:expr,) => {
        exec!($program)
    }
}

// current_path must be retrieved from the macro to
// return the crate that is tested and not this one
#[doc(hidden)]
pub fn alter_path<T: AsRef<OsStr> + ?Sized>(path: &T, current_path: &'static str) -> OsString {
    let mut paths: VecDeque<_> = split_paths(path).collect();
    let mut bins = PathBuf::from(current_path);
    bins.push("target/debug");
    paths.push_front(bins.clone());
    bins.pop();
    bins.push("release");
    paths.push_front(bins);
    join_paths(paths).expect("Invalid characters in path")
}

#[doc(hidden)]
pub fn wait(child: &mut Child, duration: Option<Duration>) -> ExitStatus {
    if let Some(duration) = duration {
        child.wait_timeout(duration).expect("Failed to wait for child process")
            .unwrap_or_else(|| {
                child.kill().expect("Failed to kill child process");
                println!("Killed child process");
                child.wait().unwrap()
            })
    } else {
        child.wait().expect("Failed to wait for child process")
    }
}

#[cfg(unix)]
#[doc(hidden)]
#[inline]
pub fn get_code(status: ExitStatus) -> i32 {
    status.code()
        .or_else(|| status.signal())
        .unwrap()
}

#[cfg(not(unix))]
#[doc(hidden)]
#[inline]
pub fn get_code(status: ExitStatus) -> i32 {
    status.code().unwrap()
}

#[doc(hidden)]
pub fn assert<T, U>(a: &T, b: &U, message: &str) where
    T: Debug + PartialEq<U>,
    U: Debug {
    if a != b {
        panic!("assertion failed: {}\nexpected `{:?}`\nfound `{:?}`", message, a, b);
    };
}

/// Input type checking, only has to compile
#[cold]
fn possible_input() {
    // byte literal
    exec! {
        "",
        stdin: b"a",
        stdout: b"b",
        stderr: b"c"
    };

    // Vec<u8>
    let buf = vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144];
    exec! {
        "",
        stdin: buf,
        stdout: vec![0x13],
        stderr: Vec::new()
    };

    // &[u8]
    let a = buf.as_slice();
    exec! {
        "",
        stdin: a,
        stdout: []
    };

    // impl AsRef<[u8]>
    struct Arbitrary<'a>(&'a [u8]);
    impl<'a> AsRef<[u8]> for Arbitrary<'a> {
        fn as_ref(&self) -> &[u8] {
            &self.0
        }
    }

    let a = Arbitrary(&buf);
    exec! {
        "",
        stdin: a,
        stdout: a,
        stderr: a
    };
}
