#![doc = include_str!("../README.md")]

//--------------------------------------------------------------------------------------------------

use bunt::termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use rayon::prelude::*;
use std::io::{Read, Write};

//--------------------------------------------------------------------------------------------------

macro_rules! cprint {
    ($color:expr, $($x:tt)*) => {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        stdout.set_color($color).unwrap();
        write!(&mut stdout, $($x)*).unwrap();
        stdout.reset().unwrap();
    };
}

macro_rules! cprintln {
    ($color:expr, $($x:tt)*) => {
        let mut stdout = StandardStream::stdout(ColorChoice::Auto);
        stdout.set_color($color).unwrap();
        write!(&mut stdout, $($x)*).unwrap();
        stdout.reset().unwrap();
        writeln!(&mut stdout).unwrap();
    };
}

//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Pipe {
    #[default]
    Null,
    Stdout,
    Stderr,
    String(String),
}

impl Pipe {
    pub fn string() -> Pipe {
        Pipe::String(String::new())
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

            let r = commands
                .iter()
                .enumerate()
                .map(|(i, command)| {
                    if i > 0 && self.print && !self.dry_run {
                        bunt::println!("");
                    }

                    self.run1(command)
                })
                .collect();

            if self.print {
                cprintln!(&self.fence_color, "{}\n", self.fence);
            }

            r
        } else {
            commands
                .par_iter()
                .map(|command| self.run1(command))
                .collect()
        }
    }

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
                    .replace("; ", " \\\n; "),
            );
        }

        if self.dry_run {
            return command.clone();
        }

        let (prog, args) = self.prepare(&command.command);

        let mut cmd = std::process::Command::new(prog);
        cmd.args(&args);

        if matches!(command.stdin, Pipe::String(_)) {
            cmd.stdin(std::process::Stdio::piped());
        }

        if matches!(command.stdout, Some(Pipe::String(_) | Pipe::Null)) {
            cmd.stdout(std::process::Stdio::piped());
        }

        if matches!(command.stderr, Some(Pipe::String(_) | Pipe::Null)) {
            cmd.stderr(std::process::Stdio::piped());
        }

        let mut child = cmd.spawn().unwrap();

        if let Pipe::String(s) = &command.stdin {
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(s.as_bytes()).unwrap();
        }

        let mut r = command.clone();

        r.code = match child.wait() {
            Ok(status) => status.code(),
            Err(_e) => None,
        };

        if matches!(command.stdout, Some(Pipe::String(_))) {
            let mut stdout = String::new();
            child.stdout.unwrap().read_to_string(&mut stdout).unwrap();
            r.stdout = Some(Pipe::String(stdout));
        }

        if matches!(command.stderr, Some(Pipe::String(_))) {
            let mut stderr = String::new();
            child.stderr.unwrap().read_to_string(&mut stderr).unwrap();
            r.stderr = Some(Pipe::String(stderr));
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
}

//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Command {
    pub command: String,
    pub stdin: Pipe,
    pub codes: Vec<i32>,

    pub stdout: Option<Pipe>,
    pub stderr: Option<Pipe>,
    pub code: Option<i32>,
}

impl Command {
    pub fn new(command: &str) -> Command {
        Command {
            command: command.to_string(),
            ..Default::default()
        }
    }
}
