//! Testing a testing tool.

#[macro_use]
extern crate test_exec;

#[test]
fn test_args() {
    exec! {
        "echo",
        args: ["Hello world!"],
        log: true,

        stdout: b"Hello world!\n",
        stderr: []
    };
}

#[test]
fn test_cwd() {
    exec! {
        "pwd",
        cwd: "/",
        log: true


    };
}

#[test]
fn test_env() {
    let output = exec! {
        "printenv",
        clear_env: true,
        env: {
            USER: "root",
            COFFEE: "a lot"
        },
        modify_path: false,
        log: true,

        stderr: []
    };
    assert!(if output.stdout == b"COFFEE=a lot\nUSER=root\n" ||
        output.stdout == b"USER=root\nCOFFEE=a lot\n" {
        true
    } else {
        false
    });
}

#[test]
fn test_modify_path() {
    let mut path = std::env::current_dir().unwrap();
    path.push("target/debug/printenv");

    exec! {
        path,
        args: ["PATH"],
        env: {
            PATH: "/bin"
        },
        modify_path: false,
        log: true,

        stdout: b"/bin\n",
        stderr: []
    };
}

#[test]
fn test_auto_modify_path() {
    exec! {
        "example_bin",
        log: true,

        stdout: b"Hello from the example binary target\n".to_vec(),
        stderr: []
    };
}

#[cfg(unix)]
#[test]
fn test_timeout() {
    exec! {
        "sleep",
        args: ["60"],
        timeout: 3000,
        log: true,

        code: 9, // SIGKILL
        stdout: [],
        stderr: []
    };
}

#[cfg(windows)]
#[test]
fn test_timeout() {
    exec! {
        "sleep",
        args: ["60"],
        timeout: 3000,
        log: true,

        code: 1,
        stdout: [],
        stderr: []
    };
}


#[test]
fn test_code(){
    exec!("true");

    exec!{
        "false",
        code: 1
    };
}

#[cfg(unix)]
#[test]
fn test_signal() {
    exec! {
        "sleep",
        args: ["60"],
        timeout: 1000,
        log: true,

        code: 9,
        stdout: [],
        stderr: [],
        signal: 9 // SIGKILL = 9
    };
}

#[cfg(unix)]
#[test]
#[should_panic]
fn test_signal_none() {
    exec! {
        "true",
        log: true,

        stdout: [],
        stderr: [],
        signal: 9
    };
}

#[cfg(not(unix))]
#[test]
fn test_signal_ignored() {
    exec!{
        "true",
        log: true,

        signal: 0xfff
    };
}

#[test]
fn test_stdin() {
    exec! {
        "print_stdin",
        stdin: b"meow",
        log: true,

        stdout: b"meow",
        stderr: []
    };
}
