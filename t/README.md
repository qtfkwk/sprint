# About

The `sprint` crate provides the [`Shell`] struct which represents a shell
session in your library or CLI code and can be used for running commands:

* [Run command(s) and show the output](#run-commands-and-show-the-output)
    * Methods
        * [`run`][`Shell::run`]
        * [`run_check`][`Shell::run_check`]
    * Options include `shell`, `fence`, `info`, `prompt`, `print`, `dry_run`

* [Run command(s) and return the output](#run-commands-and-return-the-output)
    * Methods
        * [`pipe`][`Shell::pipe`]
        * [`pipe_with`][`Shell::pipe_with`]
        * [`pipe_with1`][`Shell::pipe_with1`]
    * Options include `shell`, `sync`

[`Shell`] exposes its properties so you can easily
[create a custom shell](#customize) or [modify an existing shell](#modify) with
the settings you want.

[`Shell`]: https://docs.rs/sprint/latest/sprint/struct.Shell.html
[`Shell::run`]: https://docs.rs/sprint/latest/sprint/struct.Shell.html#method.run
[`Shell::run_check`]: https://docs.rs/sprint/latest/sprint/struct.Shell.html#method.run_check
[`Shell::pipe`]: https://docs.rs/sprint/latest/sprint/struct.Shell.html#method.pipe
[`Shell::pipe_with`]: https://docs.rs/sprint/latest/sprint/struct.Shell.html#method.pipe_with
[`Shell::pipe_with1`]: https://docs.rs/sprint/latest/sprint/struct.Shell.html#method.pipe_with1

# Examples

## Run command(s) and show the output

~~~rust
use sprint::*;

let shell = Shell::default();

shell.run(&["ls", "ls -l"]).unwrap();
~~~

## Run command(s) and return the output

~~~rust
use sprint::*;

let shell = Shell::default();

assert_eq!(
    shell.pipe(&["ls"]),
    vec![(
      String::from(
          "\\
Cargo.lock
Cargo.toml
CHANGELOG.md
Makefile.md
README.md
src
t
target
tests
\\
          ",
      ),
      String::default(),
      Some(0),
    )],
);
~~~

## Customize

~~~rust
use sprint::*;

let shell = Shell {
    // Common options

    // Shell
    shell: Some(String::from("sh -c")),
    //shell: None, // Run directly

    // ---

    // Options for run, run_check

    // Print extra content

    fence: String::from("```"),
    info: String::from("text"),
    prompt: String::from("$ "),

    fence_color: bunt::style!("#555555"),
    info_color: bunt::style!("#555555"),
    prompt_color: bunt::style!("#555555"),
    command_color: bunt::style!("#00ffff+bold"),

    print: true,

    // Don't run command(s)
    dry_run: false,
    //dry_run: true,

    // ---

    // Options for pipe, pipe_with

    // Run commands synchronously
    sync: true,
    //sync: false,
};

shell.run(&["ls", "ls -l"]).unwrap();
~~~

## Modify

~~~rust
use sprint::*;

let mut shell = Shell::default();

shell.shell = None;
shell.sync = false;

let results = shell.pipe(&["ls", "ls -l"]);
~~~

!inc:../CHANGELOG.md

