use sprint::*;

const COMMANDS: &[&str] = &["ls *", "ls -l"];

#[test]
fn default() {
    let shell = Shell::default();

    shell.run(COMMANDS).unwrap();
}

#[test]
fn manual() {
    let shell = Shell {
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
    };

    shell.run(COMMANDS).unwrap();
}

#[test]
fn custom() {
    let shell = Shell {
        shell: Some(String::from("bash -xeo pipefail -c")),

        fence: String::from("~~~~"),
        info: String::from("bash"),
        prompt: String::from("> "),

        fence_color: bunt::style!("#ffff00"),
        info_color: bunt::style!("#ff0000+italic"),
        prompt_color: bunt::style!("#00ff00"),
        command_color: bunt::style!("#ff00ff+bold"),

        print: true,
        dry_run: false,
        sync: true,
    };

    shell.run(COMMANDS).unwrap();
}

#[test]
fn direct() {
    let shell = Shell {
        shell: None,

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
    };

    shell.run_check(&[("ls *", &[2]), ("ls -l", &[0])]).unwrap();
}

#[test]
fn pipe() {
    assert_eq!(
        Shell::default().pipe(&["ls"]),
        vec![(
            String::from(
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
            ),
            String::new(),
            Some(0),
        )],
    );
}

#[test]
fn pipe_with() {
    assert_eq!(
        Shell::default().pipe_with(&[("ls", None)]),
        vec![(
            String::from(
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
            ),
            String::new(),
            Some(0),
        )],
    );
}

#[test]
fn pipe_sync() {
    assert_eq!(
        Shell::default().pipe(&["ls", "ls C*"]),
        vec![
            (
                String::from(
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
                ),
                String::new(),
                Some(0),
            ),
            (
                String::from(
                    "\
Cargo.lock
Cargo.toml
CHANGELOG.md
\
                    ",
                ),
                String::new(),
                Some(0),
            ),
        ],
    );
}

#[test]
fn pipe_async() {
    assert_eq!(
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
            sync: false,
        }
        .pipe(&["ls", "ls C*"]),
        vec![
            (
                String::from(
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
                ),
                String::new(),
                Some(0),
            ),
            (
                String::from(
                    "\
Cargo.lock
Cargo.toml
CHANGELOG.md
\
                    ",
                ),
                String::new(),
                Some(0),
            ),
        ],
    );
}
