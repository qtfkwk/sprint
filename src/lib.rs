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
    color: ColorOverride::Auto,

    fence: String::from("```"),
    info: String::from("text"),
    prompt: String::from("$ "),

    fence_style: style("#555555").expect("style"),
    info_style: style("#555555").expect("style"),
    prompt_style: style("#555555").expect("style"),
    command_style: style("#00ffff+bold").expect("style"),
    error_style: style("#ff0000+bold+italic").expect("style"),
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

use {
    anstream::{print, println},
    anyhow::{anyhow, Result},
    clap::ValueEnum,
    owo_colors::{OwoColorize, Rgb, Style},
    rayon::prelude::*,
    std::io::{Read, Write},
};

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

/// Create a [`Style`] from a [`&str`] specification
pub fn style(s: &str) -> Result<Style> {
    let mut r = Style::new();
    for i in s.split('+') {
        if let Some(color) = i.strip_prefix('#') {
            r = r.color(html(color)?);
        } else if let Some(color) = i.strip_prefix("on-#") {
            r = r.on_color(html(color)?);
        } else {
            match i {
                "black" => r = r.black(),
                "red" => r = r.red(),
                "green" => r = r.green(),
                "yellow" => r = r.yellow(),
                "blue" => r = r.blue(),
                "magenta" => r = r.magenta(),
                "purple" => r = r.purple(),
                "cyan" => r = r.cyan(),
                "white" => r = r.white(),
                //---
                "bold" => r = r.bold(),
                "italic" => r = r.italic(),
                "dimmed" => r = r.dimmed(),
                "underline" => r = r.underline(),
                "blink" => r = r.blink(),
                "blink_fast" => r = r.blink_fast(),
                "reversed" => r = r.reversed(),
                "hidden" => r = r.hidden(),
                "strikethrough" => r = r.strikethrough(),
                //---
                "bright-black" => r = r.bright_black(),
                "bright-red" => r = r.bright_red(),
                "bright-green" => r = r.bright_green(),
                "bright-yellow" => r = r.bright_yellow(),
                "bright-blue" => r = r.bright_blue(),
                "bright-magenta" => r = r.bright_magenta(),
                "bright-purple" => r = r.bright_purple(),
                "bright-cyan" => r = r.bright_cyan(),
                "bright-white" => r = r.bright_white(),
                //---
                "on-black" => r = r.on_black(),
                "on-red" => r = r.on_red(),
                "on-green" => r = r.on_green(),
                "on-yellow" => r = r.on_yellow(),
                "on-blue" => r = r.on_blue(),
                "on-magenta" => r = r.on_magenta(),
                "on-purple" => r = r.on_purple(),
                "on-cyan" => r = r.on_cyan(),
                "on-white" => r = r.on_white(),
                //---
                "on-bright-black" => r = r.on_bright_black(),
                "on-bright-red" => r = r.on_bright_red(),
                "on-bright-green" => r = r.on_bright_green(),
                "on-bright-yellow" => r = r.on_bright_yellow(),
                "on-bright-blue" => r = r.on_bright_blue(),
                "on-bright-magenta" => r = r.on_bright_magenta(),
                "on-bright-purple" => r = r.on_bright_purple(),
                "on-bright-cyan" => r = r.on_bright_cyan(),
                "on-bright-white" => r = r.on_bright_white(),
                //---
                _ => return Err(anyhow!("Invalid style spec: {s:?}!")),
            }
        }
    }
    Ok(r)
}

fn html(rrggbb: &str) -> Result<Rgb> {
    let r = u8::from_str_radix(&rrggbb[0..2], 16)?;
    let g = u8::from_str_radix(&rrggbb[2..4], 16)?;
    let b = u8::from_str_radix(&rrggbb[4..6], 16)?;
    Ok(Rgb(r, g, b))
}

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum ColorOverride {
    #[default]
    Auto,
    Always,
    Never,
}

impl ColorOverride {
    pub fn init(&self) {
        match self {
            ColorOverride::Always => anstream::ColorChoice::Always.write_global(),
            ColorOverride::Never => anstream::ColorChoice::Never.write_global(),
            ColorOverride::Auto => {}
        }
    }
}

//--------------------------------------------------------------------------------------------------

struct Prefix {
    style: Style,
}

impl std::fmt::Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.style.fmt_prefix(f)
    }
}

fn print_prefix(style: Style) {
    print!("{}", Prefix { style });
}

//--------------------------------------------------------------------------------------------------

struct Suffix {
    style: Style,
}

impl std::fmt::Display for Suffix {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.style.fmt_suffix(f)
    }
}

fn print_suffix(style: Style) {
    print!("{}", Suffix { style });
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
    color: ColorOverride::default(),

    fence: String::from("```"),
    info: String::from("text"),
    prompt: String::from("$ "),

    fence_style: style("#555555").expect("style"),
    info_style: style("#555555").expect("style"),
    prompt_style: style("#555555").expect("style"),
    command_style: style("#00ffff+bold").expect("style"),
    error_style: style("#ff0000+bold+italic").expect("style"),
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
    pub color: ColorOverride,

    pub fence: String,
    pub info: String,
    pub prompt: String,

    pub fence_style: Style,
    pub info_style: Style,
    pub prompt_style: Style,
    pub command_style: Style,
    pub error_style: Style,
}

impl Default for Shell {
    /// Default [`Shell`]
    fn default() -> Shell {
        Shell {
            shell: Some(String::from("sh -c")),

            dry_run: false,
            sync: true,
            print: true,
            color: ColorOverride::default(),

            fence: String::from("```"),
            info: String::from("text"),
            prompt: String::from("$ "),

            fence_style: style("#555555").expect("style"),
            info_style: style("#555555").expect("style"),
            prompt_style: style("#555555").expect("style"),
            command_style: style("#00ffff+bold").expect("style"),
            error_style: style("#ff0000+bold+italic").expect("style"),
        }
    }
}

impl Shell {
    /// Run command(s)
    pub fn run(&self, commands: &[Command]) -> Vec<Command> {
        if self.sync {
            if self.print {
                self.print_fence(0);
                println!("{}", self.info.style(self.info_style));
            }

            let mut r = vec![];
            let mut error = None;

            for (i, command) in commands.iter().enumerate() {
                if i > 0 && self.print && !self.dry_run {
                    println!();
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
                self.print_fence(2);

                if let Some(error) = error {
                    println!("{}\n", error.style(self.error_style));
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

    /// Run a single command
    pub fn run1(&self, command: &Command) -> Command {
        if self.print {
            if !self.dry_run {
                print!("{}", self.prompt.style(self.prompt_style));
            }

            println!(
                "{}",
                command
                    .command
                    .replace(" && ", " \\\n&& ")
                    .replace(" || ", " \\\n|| ")
                    .replace("; ", "; \\\n")
                    .style(self.command_style),
            );
        }

        if self.dry_run {
            return command.clone();
        }

        self.core(command)
    }

    /// Pipe a single command
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

    /// Run a command in a child process
    pub fn run1_async(&self, command: &Command) -> std::process::Child {
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
                self.print_fence(0);
                println!("{}", command.command.style(self.info_style));
                println!("{s}");
                self.print_fence(2);
                self.print_fence(0);
                println!("{}", self.info.style(self.info_style));
            }
        }

        let mut child = cmd.spawn().unwrap();

        if let Pipe::String(Some(s)) = &command.stdin {
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(s.as_bytes()).unwrap();
        }

        child
    }

    /// Core part to run/pipe a command
    pub fn core(&self, command: &Command) -> Command {
        let mut child = self.run1_async(command);

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
                self.print_fence(2);
            }
        }

        r
    }

    /// Prepare the command
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

    /// Print the fence
    pub fn print_fence(&self, newlines: usize) {
        print!(
            "{}{}",
            self.fence.style(self.fence_style),
            "\n".repeat(newlines),
        );
    }

    /// Print the interactive prompt
    pub fn interactive_prompt(&self, previous: bool) {
        if previous {
            self.print_fence(2);
        }

        self.print_fence(0);
        println!("{}", self.info.style(self.info_style));
        print!("{}", self.prompt.style(self.prompt_style));

        // Set the command style
        print_prefix(self.command_style);
        std::io::stdout().flush().expect("flush");
    }

    /// Clear the command style
    pub fn interactive_prompt_reset(&self) {
        print_suffix(self.command_style);
        std::io::stdout().flush().expect("flush");
    }

    /// Simpler interface to run command(s)
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
