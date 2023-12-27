use sprint::*;

#[test]
fn default() {
    let shell = Shell::default();

    shell.run(&[Command::new("ls *"), Command::new("ls -l")]);
}

#[test]
fn manual() {
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

    shell.run(&[Command::new("ls *"), Command::new("ls -l")]);
}

#[test]
fn custom() {
    let shell = Shell {
        shell: Some(String::from("bash -xeo pipefail -c")),

        dry_run: false,
        sync: true,
        print: true,

        fence: String::from("~~~~"),
        info: String::from("bash"),
        prompt: String::from("> "),

        fence_color: bunt::style!("#ffff00"),
        info_color: bunt::style!("#ff0000+italic"),
        prompt_color: bunt::style!("#00ff00"),
        command_color: bunt::style!("#ff00ff+bold"),
        error_color: bunt::style!("#00ff00+bold+italic"),
    };

    shell.run(&[Command::new("ls *"), Command::new("ls -l")]);
}

#[test]
fn direct() {
    let shell = Shell {
        shell: None,

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

    shell.run(&[
        Command {
            command: String::from("ls *"),
            codes: vec![2],
            ..Default::default()
        },
        Command {
            command: String::from("ls -l"),
            ..Default::default()
        },
    ]);
}

#[test]
fn pipe() {
    assert_eq!(
        Shell {
            print: false,
            ..Default::default()
        }
        .run(&[Command {
            command: String::from("ls"),
            stdout: Some(Pipe::string()),
            codes: vec![0],
            ..Default::default()
        }])[0]
            .stdout,
        Some(Pipe::String(String::from(
            "\
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
            "
        ))),
    );
}
