use sprint::*;

#[test]
fn default() {
    println!();

    let shell = Shell::default();

    shell.run(&[Command::new("ls *"), Command::new("ls -l")]);
}

#[test]
fn manual() {
    println!();

    let shell = Shell {
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
    };

    shell.run(&[Command::new("ls *"), Command::new("ls -l")]);
}

#[test]
fn custom() {
    println!();

    let shell = Shell {
        shell: Some(String::from("bash -xeo pipefail -c")),

        dry_run: false,
        sync: true,
        print: true,
        color: ColorOverride::default(),

        fence: String::from("~~~~"),
        info: String::from("bash"),
        prompt: String::from("> "),

        fence_style: style("#ffff00").expect("style"),
        info_style: style("#ff0000+italic").expect("style"),
        prompt_style: style("#00ff00").expect("style"),
        command_style: style("#ff00ff+bold").expect("style"),
        error_style: style("#00ff00+bold+italic").expect("style"),
    };

    shell.run(&[Command::new("ls *"), Command::new("ls -l")]);
}

#[test]
fn direct() {
    println!();

    let shell = Shell {
        shell: None,

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
fn pipe_manual() {
    assert_eq!(
        Shell {
            print: false,
            ..Default::default()
        }
        .run(&[Command {
            command: String::from("ls"),
            stdout: Pipe::string(),
            codes: vec![0],
            ..Default::default()
        }])[0]
            .stdout,
        Pipe::String(Some(String::from(
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

#[test]
fn pipe1() {
    assert_eq!(
        Shell::default().pipe1("ls"),
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
    );
}
