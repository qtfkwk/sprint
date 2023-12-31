/*!
# About

The `sprint` crate provides the [`Shell`] struct which represents a shell
session in your library or CLI code and can be used for running commands:

* [Show the output](#run-commands-and-show-the-output)
* [Return the output](#run-commands-and-return-the-output)

[`Shell`] exposes its properties so you can easily
[create a custom shell](#customize) or [modify an existing shell](#modify) with
the settings you want.

[`Shell`]: https://docs.rs/sprint/latest/sprint/struct.Shell.html

# Examples

## Run command(s) and show the output

~~~rust
use sprint::*;

let shell = Shell::default();

shell.run(&[Command::new("ls"), Command::new("ls -l")]);

// or equivalently:
//shell.run_str(&["ls", "ls -l"]);
~~~

## Run command(s) and return the output

~~~rust
use sprint::*;

let shell = Shell::default();

let results = shell.run(&[Command {
    command: String::from("ls"),
    stdout: Pipe::string(),
    codes: vec![0],
    ..Default::default()
}]);

assert_eq!(
    results[0].stdout,
    Pipe::String(Some(String::from("\
Cargo.lock
Cargo.toml
CHANGELOG.md
Makefile.md
README.md
src
t
target
tests
\
    "))),
);
~~~

## Customize

~~~rust
use sprint::*;

let shell = Shell {
    shell: Some(String::from("sh -c")),

    dry_run: false,
    sync: true,
    print: true,

    fence: String::from("```"),
    info: String::from("text"),
    prompt: String::from("$ "),

    fence_color: bunt::style!("#555555"),
    info_color: bunt::style!("#555555"),
    prompt_color: bunt::style!("#555555"),
    command_color: bunt::style!("#00ffff+bold"),
    error_color: bunt::style!("#ff0000+bold+italic"),
};

shell.run(&[Command::new("ls"), Command::new("ls -l")]);
~~~

## Modify

~~~rust
use sprint::*;

let mut shell = Shell::default();

shell.shell = None;

shell.run(&[Command::new("ls"), Command::new("ls -l")]);

shell.sync = false;

shell.run(&[Command::new("ls"), Command::new("ls -l")]);
~~~
*/

//--------------------------------------------------------------------------------------------------

use bunt::termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use rayon::prelude::*;
use std::io::{IsTerminal, Read, Write};

//--------------------------------------------------------------------------------------------------

/**
Get stdout as a [`StandardStream`]
*/
pub fn get_stdout() -> StandardStream {
    StandardStream::stdout(if std::io::stdout().is_terminal() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    })
}

/**
Clear the color style
*/
pub fn reset_stdout() {
    let mut stdout = get_stdout();
    stdout.reset().unwrap();
    stdout.flush().unwrap();
}

macro_rules! cprint {
    ($color:expr, $($x:tt)*) => {
        let mut stdout = get_stdout();
        stdout.set_color($color).unwrap();
        write!(&mut stdout, $($x)*).unwrap();
        stdout.reset().unwrap();
        stdout.flush().unwrap();
    };
}

macro_rules! cprintln {
    ($color:expr, $($x:tt)*) => {
        let mut stdout = get_stdout();
        stdout.set_color($color).unwrap();
        write!(&mut stdout, $($x)*).unwrap();
        stdout.reset().unwrap();
        writeln!(&mut stdout).unwrap();
        stdout.flush().unwrap();
    };
}

//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pipe {
    Null,
    Stdout,
    Stderr,
    String(Option<String>),
}

impl Pipe {
    pub fn string() -> Pipe {
        Pipe::String(None)
    }
}

//--------------------------------------------------------------------------------------------------

/**
Command runner

*Please see also the module-level documentation for a high-level description and examples.*

```
use sprint::*;

// Use the default configuration:

let shell = Shell::default();

// Or a custom configuration:

let shell = Shell {
    shell: Some(String::from("sh -c")),
    //shell: Some(String::from("bash -c")), // Use bash
    //shell: Some(String::from("bash -xeo pipefail -c")), // Use bash w/ options
    //shell: None, // Run directly instead of a shell

    dry_run: false,
    sync: true,
    print: true,

    fence: String::from("```"),
    info: String::from("text"),
    prompt: String::from("$ "),

    fence_color: bunt::style!("#555555"),
    info_color: bunt::style!("#555555"),
    prompt_color: bunt::style!("#555555"),
    command_color: bunt::style!("#00ffff+bold"),
    error_color: bunt::style!("#ff0000+bold+italic"),
};

// Or modify it on the fly:

let mut shell = Shell::default();

shell.shell = None;
shell.sync = false;

// ...
```
*/
#[derive(Clone, Debug)]
pub struct Shell {
    pub shell: Option<String>,

    pub dry_run: bool,
    pub sync: bool,
    pub print: bool,

    pub fence: String,
    pub info: String,
    pub prompt: String,

    pub fence_color: ColorSpec,
    pub info_color: ColorSpec,
    pub prompt_color: ColorSpec,
    pub command_color: ColorSpec,
    pub error_color: ColorSpec,
}

impl Default for Shell {
    /**
    Default [`Shell`]
    */
    fn default() -> Shell {
        Shell {
            shell: Some(String::from("sh -c")),

            dry_run: false,
            sync: true,
            print: true,

            fence: String::from("```"),
            info: String::from("text"),
            prompt: String::from("$ "),

            fence_color: bunt::style!("#555555"),
            info_color: bunt::style!("#555555"),
            prompt_color: bunt::style!("#555555"),
            command_color: bunt::style!("#00ffff+bold"),
            error_color: bunt::style!("#ff0000+bold+italic"),
        }
    }
}

impl Shell {
    /**
    Run command(s)
    */
    pub fn run(&self, commands: &[Command]) -> Vec<Command> {
        if self.sync {
            if self.print {
                cprint!(&self.fence_color, "{}", self.fence);
                cprintln!(&self.info_color, "{}", self.info);
            }

            let mut r = vec![];
            let mut error = None;

            for (i, command) in commands.iter().enumerate() {
                if i > 0 && self.print && !self.dry_run {
                    bunt::println!("");
                }

                let result = self.run1(command);

                if let Some(code) = &result.code {
                    if !result.codes.contains(code) {
                        error = Some(format!(
                            "**Command `{}` exited with code: `{code}`!**",
                            result.command,
                        ));
                    }
                } else if !self.dry_run {
                    error = Some(format!(
                        "**Command `{}` was killed by a signal!**",
                        result.command,
                    ));
                }

                r.push(result);

                if error.is_some() {
                    break;
                }
            }

            if self.print {
                cprintln!(&self.fence_color, "{}\n", self.fence);

                if let Some(error) = error {
                    cprintln!(&self.error_color, "{}\n", error);
                }
            }

            r
        } else {
            commands
                .par_iter()
                .map(|command| self.run1(command))
                .collect()
        }
    }

    /**
    Run a single command
    */
    pub fn run1(&self, command: &Command) -> Command {
        if self.print {
            if !self.dry_run {
                cprint!(&self.prompt_color, "{}", self.prompt);
            }
            cprintln!(
                &self.command_color,
                "{}",
                command
                    .command
                    .replace(" && ", " \\\n&& ")
                    .replace(" || ", " \\\n|| ")
                    .replace("; ", "; \\\n"),
            );
        }

        if self.dry_run {
            return command.clone();
        }

        self.core(command)
    }

    /**
    Pipe a single command
    */
    pub fn pipe1(&self, command: &str) -> String {
        let command = Command {
            command: command.to_string(),
            stdout: Pipe::string(),
            ..Default::default()
        };

        let result = self.core(&command);

        if let Pipe::String(Some(stdout)) = &result.stdout {
            stdout.to_string()
        } else {
            String::new()
        }
    }

    /**
    Core part to run/pipe a command
    */
    pub fn core(&self, command: &Command) -> Command {
        let (prog, args) = self.prepare(&command.command);

        let mut cmd = std::process::Command::new(prog);
        cmd.args(&args);

        if matches!(command.stdin, Pipe::String(_)) {
            cmd.stdin(std::process::Stdio::piped());
        }

        if matches!(command.stdout, Pipe::String(_) | Pipe::Null) {
            cmd.stdout(std::process::Stdio::piped());
        }

        if matches!(command.stderr, Pipe::String(_) | Pipe::Null) {
            cmd.stderr(std::process::Stdio::piped());
        }

        if self.print {
            if let Pipe::String(Some(s)) = &command.stdin {
                cprint!(&self.fence_color, "{}", self.fence);
                cprintln!(&self.info_color, "{}", command.command);
                println!("{s}");
                cprintln!(&self.fence_color, "{}\n", self.fence);
                cprint!(&self.fence_color, "{}", self.fence);
                cprintln!(&self.info_color, "{}", self.info);
            }
        }

        let mut child = cmd.spawn().unwrap();

        if let Pipe::String(Some(s)) = &command.stdin {
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(s.as_bytes()).unwrap();
        }

        let mut r = command.clone();

        r.code = match child.wait() {
            Ok(status) => status.code(),
            Err(_e) => None,
        };

        if matches!(command.stdout, Pipe::String(_)) {
            let mut stdout = String::new();
            child.stdout.unwrap().read_to_string(&mut stdout).unwrap();
            r.stdout = Pipe::String(Some(stdout));
        }

        if matches!(command.stderr, Pipe::String(_)) {
            let mut stderr = String::new();
            child.stderr.unwrap().read_to_string(&mut stderr).unwrap();
            r.stderr = Pipe::String(Some(stderr));
        }

        if self.print {
            if let Pipe::String(Some(_s)) = &command.stdin {
                cprintln!(&self.fence_color, "{}\n", self.fence);
            }
        }

        r
    }

    /**
    Prepare the command
    */
    fn prepare(&self, command: &str) -> (String, Vec<String>) {
        if let Some(s) = &self.shell {
            let mut args = shlex::split(s).unwrap();
            let prog = args.remove(0);
            args.push(command.to_string());
            (prog, args)
        } else {
            // Shell disabled; run command directly
            let mut args = shlex::split(command).unwrap();
            let prog = args.remove(0);
            (prog, args)
        }
    }

    /**
    Print the interactive prompt
    */
    pub fn interactive_prompt(&self, previous: bool) {
        if previous {
            cprintln!(&self.fence_color, "{}\n", self.fence);
        }
        cprint!(&self.fence_color, "{}", self.fence);
        cprintln!(&self.info_color, "{}", self.info);
        cprint!(&self.prompt_color, "{}", self.prompt);

        // Set the command color
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        stdout.set_color(&self.command_color).unwrap();
        stdout.flush().unwrap();
    }

    /**
    Simpler interface to run command(s)
    */
    pub fn run_str(&self, commands: &[&str]) -> Vec<Command> {
        self.run(&commands.iter().map(|x| Command::new(x)).collect::<Vec<_>>())
    }
}

//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub command: String,
    pub stdin: Pipe,
    pub codes: Vec<i32>,
    pub stdout: Pipe,
    pub stderr: Pipe,
    pub code: Option<i32>,
}

impl Default for Command {
    fn default() -> Command {
        Command {
            command: Default::default(),
            stdin: Pipe::Null,
            codes: vec![0],
            stdout: Pipe::Stdout,
            stderr: Pipe::Stderr,
            code: Default::default(),
        }
    }
}

impl Command {
    pub fn new(command: &str) -> Command {
        Command {
            command: command.to_string(),
            ..Default::default()
        }
    }
}
