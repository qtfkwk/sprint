#![doc = include_str!("../README.md")]

use anyhow::{anyhow, Result};
use bunt::termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use rayon::prelude::*;
use std::io::{Read, Write};

macro_rules! error {
    ($($x:tt)*) => { Err(anyhow!(format!($($x)*))) };
}

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

/**
Command runner

*Please see also the module-level documentation for a high-level description and examples.*

```
use sprint::*;

// Use the default configuration:

let shell = Shell::default();

// Or a custom configuration:

let shell = Shell {
    // Common options

    // Shell
    shell: Some(String::from("sh -c")),
    //shell: Some(String::from("bash -c")), // Use bash
    //shell: Some(String::from("bash -xeo pipefail -c")), // Use bash w/ options
    //shell: None, // Run directly instead of a shell

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

// Or modify it on the fly:

let mut shell = Shell::default();

shell.shell = None;
shell.sync = false;

// ...
```
*/
#[derive(Clone)]
pub struct Shell {
    pub shell: Option<String>,

    pub fence: String,
    pub info: String,
    pub prompt: String,

    pub fence_color: ColorSpec,
    pub info_color: ColorSpec,
    pub prompt_color: ColorSpec,
    pub command_color: ColorSpec,

    pub print: bool,
    pub dry_run: bool,
    pub sync: bool,
}

impl Default for Shell {
    /**
    Default [`Shell`]
    */
    fn default() -> Shell {
        Shell {
            shell: Some(String::from("sh -c")),

            fence: String::from("```"),
            info: String::from("text"),
            prompt: String::from("$ "),

            fence_color: bunt::style!("#555555"),
            info_color: bunt::style!("#555555"),
            prompt_color: bunt::style!("#555555"),
            command_color: bunt::style!("#00ffff+bold"),

            print: true,
            dry_run: false,
            sync: true,
        }
    }
}

impl Shell {
    /**
    Run command(s) and print the output

    Shorthand for [`run_check`][`Shell::run_check`] to run command(s) expecting an exit code of 0.
    */
    pub fn run(&self, commands: &[&str]) -> Result<()> {
        let zero = [0].into_iter().collect::<Vec<i32>>();
        self.run_check(
            &commands
                .iter()
                .map(|x| (*x, zero.as_slice()))
                .collect::<Vec<_>>(),
        )
    }

    /**
    Run command(s) and print the output and check exit code(s)

    * Customize the `fence`, `info`, and `prompt` string properties as desired.
    * The `sync` property is not used.

    | `print` | `dry_run` | Description                                                            |
    |---------|-----------|------------------------------------------------------------------------|
    | true    | false     | This is the default. Print markdown code block and run command(s).     |
    | true    | true      | This is a dry run. Print markdown code block but don't run command(s). |
    | false   | false     | Don't print markdown code block but run command(s).                    |
    | false   | true      | This is a *null operation*; it doesn't print or run anything.          |
    */
    pub fn run_check(&self, commands: &[(&str, &[i32])]) -> Result<()> {
        // Print the starting fence and info string
        if self.print {
            cprint!(&self.fence_color, "{}", self.fence);
            cprintln!(&self.info_color, "{}", self.info);
        }

        // Iterate commands
        for (i, (command, codes)) in commands.iter().enumerate() {
            if i > 0 && self.print && !self.dry_run {
                bunt::println!("");
            }

            // Print the prompt and/or command
            if self.print {
                if !self.dry_run {
                    cprint!(&self.prompt_color, "{}", self.prompt);
                }
                cprintln!(
                    &self.command_color,
                    "{}",
                    command
                        .replace(" && ", " \\\n&& ")
                        .replace(" || ", " \\\n|| ")
                        .replace("; ", " \\\n; "),
                );
            }

            // Run the command
            if !self.dry_run {
                let (prog, args) = self.prepare(command);
                match std::process::Command::new(prog)
                    .args(&args)
                    .spawn()
                    .unwrap()
                    .wait()
                    .unwrap()
                    .code()
                {
                    Some(code) => {
                        if !codes.is_empty() && !codes.contains(&code) {
                            return error!("The command `{command}` exited with code {code}!");
                        }
                    }
                    None => {
                        return error!("The command `{command}` was terminated by a signal!");
                    }
                }
            }
        }

        // Print the ending fence
        if self.print {
            cprintln!(&self.fence_color, "{}\n", self.fence);
        }

        Ok(())
    }

    /**
    Run command(s) and return the output

    Shorthand to run command(s) that don't have stdin.
    See [`pipe_with`][`Shell::pipe_with`] or [`pipe_with1`][`Shell::pipe_with1`] for running
    command(s) that have stdin.

    ```
    use sprint::*;

    let shell = Shell::default();

    let commands = vec!["ls", "ls -l"];

    commands
        .iter()
        .zip(shell.pipe(&commands))
        .for_each(|(command, (stdout, stderr, code))| {
            // ...
        });
    ```
    */
    pub fn pipe(&self, commands: &[&str]) -> Vec<(String, String, Option<i32>)> {
        self.pipe_with(&commands.iter().map(|x| (*x, None)).collect::<Vec<_>>())
    }

    /**
    Run command(s) with optional stdin and return the output

    | `sync` | Description                                                                                       |
    |--------|---------------------------------------------------------------------------------------------------|
    | true   | Run command(s) sequentially. This is the default.                                                 |
    | false  | Run command(s) asynchronously via [`par_iter`][`rayon::iter::IntoParallelRefIterator::par_iter`]. |

    ```
    use sprint::*;

    let shell = Shell::default();

    let commands = vec![("ls", None), ("ls -l", None)];

    commands
        .iter()
        .zip(shell.pipe_with(&commands))
        .for_each(|(command, (stdout, stderr, code))| {
            // ...
        });
    ```
    */
    pub fn pipe_with(
        &self,
        commands: &[(&str, Option<&str>)],
    ) -> Vec<(String, String, Option<i32>)> {
        if self.sync {
            commands
                .iter()
                .map(|(command, stdin)| self.pipe_with1(command, *stdin))
                .collect()
        } else {
            commands
                .par_iter()
                .map(|(command, stdin)| self.pipe_with1(command, *stdin))
                .collect()
        }
    }

    /**
    Run a single command with optional stdin and return the output

    ```
    use sprint::*;

    let shell = Shell::default();

    let (stdout, stderr, code) = shell.pipe_with1("ls", None);
    ```
    */
    pub fn pipe_with1(&self, command: &str, stdin: Option<&str>) -> (String, String, Option<i32>) {
        let (prog, args) = self.prepare(command);

        let mut child = std::process::Command::new(prog)
            .args(&args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();

        if let Some(s) = stdin {
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(s.as_bytes()).unwrap();
        }

        let code = match child.wait() {
            Ok(status) => status.code(),
            Err(_e) => None,
        };

        let mut stdout = String::new();
        child.stdout.unwrap().read_to_string(&mut stdout).unwrap();

        let mut stderr = String::new();
        child.stderr.unwrap().read_to_string(&mut stderr).unwrap();

        (stdout, stderr, code)
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
