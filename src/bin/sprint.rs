use {
    anstream::println,
    anyhow::Result,
    clap::{builder::Styles, Parser},
    ignore_check::Ignore,
    notify::{
        event::{AccessKind, AccessMode},
        Event, EventKind, RecursiveMode, Watcher,
    },
    sprint::*,
    std::{
        collections::BTreeMap,
        path::{Path, PathBuf},
        thread::sleep,
        time::Duration,
    },
};

const STYLES: Styles = Styles::styled()
    .header(clap_cargo::style::HEADER)
    .usage(clap_cargo::style::USAGE)
    .literal(clap_cargo::style::LITERAL)
    .placeholder(clap_cargo::style::PLACEHOLDER)
    .error(clap_cargo::style::ERROR)
    .valid(clap_cargo::style::VALID)
    .invalid(clap_cargo::style::INVALID);

#[derive(Parser)]
#[command(about, version, max_term_width = 80, styles = STYLES)]
struct Cli {
    /// File(s) or command(s)
    #[arg(value_name = "STRING")]
    arguments: Vec<String>,

    /// Shell
    #[arg(short, long, value_name = "STRING", default_value = "sh -c")]
    shell: String,

    /// Fence
    #[arg(short, long, value_name = "STRING", default_value = "```")]
    fence: String,

    /// Info
    #[arg(short, long, value_name = "STRING", default_value = "text")]
    info: String,

    /// Prompt
    #[arg(short, long, value_name = "STRING", default_value = "$ ")]
    prompt: String,

    /// Watch files/directories and rerun command on change; see also `-d` option
    #[arg(short, long, value_name = "PATH")]
    watch: Vec<PathBuf>,

    /// Debounce; used only with `-w`
    #[arg(short, long, value_name = "SECONDS", default_value = "5.0")]
    debounce: f32,

    /// Force enable/disable terminal colors
    #[arg(short = 'C', long, default_value = "auto")]
    color: ColorOverride,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    cli.color.init();

    let shell = Shell {
        shell: Some(cli.shell.clone()),
        fence: cli.fence.clone(),
        info: cli.info.clone(),
        prompt: cli.prompt.clone(),
        ..Default::default()
    };

    let no_arguments = cli.arguments.is_empty();
    let no_watch = cli.watch.is_empty();

    if no_arguments && no_watch {
        // Run interactively

        let stdin = std::io::stdin();
        shell.interactive_prompt(false);
        loop {
            let mut command = String::new();
            if stdin.read_line(&mut command).is_ok() {
                shell.interactive_prompt_reset();

                if command.is_empty() {
                    // Control + D
                    break;
                }

                let result = shell.core(&Command::new(command.trim()));

                if let Some(code) = &result.code {
                    if !result.codes.contains(code) {
                        std::process::exit(*code);
                    }
                } else {
                    std::process::exit(1);
                }

                shell.interactive_prompt(true);
            } else {
                std::process::exit(1);
            }
        }
    } else if no_watch {
        // Run given commands / files

        let results = shell.run(
            &cli.arguments
                .iter()
                .map(|x| Command::new(x))
                .collect::<Vec<_>>(),
        );

        // Exit with the code of the last command
        std::process::exit(results.last().unwrap().code.unwrap_or(1));
    } else if no_arguments {
        // Watch, but no commands...

        // Get watched directories & files
        let (dirs, mut hashes) = watched(&cli.watch);
        let ignored = Ignore::default();
        let pwd = std::env::current_dir().unwrap();

        let debounce = std::time::Duration::from_secs_f32(cli.debounce);
        let mut ts = std::time::Instant::now();

        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    let now = std::time::Instant::now();
                    match event.kind {
                        EventKind::Create(_) | EventKind::Remove(_) => {
                            // Created or deleted a file/directory
                            'outer: for path in event
                                .paths
                                .iter()
                                .map(|x| x.strip_prefix(&pwd).unwrap().to_path_buf())
                                .filter(|x| not_ignored(x, &ignored, &dirs, &hashes))
                            {
                                if now - ts > debounce {
                                    println!(
                                        "* {}: `{}`",
                                        match event.kind {
                                            EventKind::Create(_) => "Created",
                                            EventKind::Remove(_) => "Removed",
                                            _ => unreachable!(),
                                        },
                                        path.display(),
                                    );
                                    ts = now;
                                    break 'outer;
                                }
                            }
                        }
                        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                            // Wrote a file
                            let mut not_restarted = true;
                            let paths = event
                                .paths
                                .iter()
                                .map(|x| x.strip_prefix(&pwd).unwrap().to_path_buf())
                                .filter(|x| not_ignored(x, &ignored, &dirs, &hashes))
                                .collect::<Vec<_>>();
                            for path in paths {
                                if let Some(h1) = hashes.get(&path) {
                                    let h2 = hash(&path);
                                    if h2 != *h1 {
                                        // File changed...

                                        // Update the hash
                                        hashes.insert(path.clone(), h2);

                                        if not_restarted && now - ts > debounce {
                                            println!("* Modified: `{}`", path.display());
                                            ts = now;
                                            not_restarted = false;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(_e) => {
                    std::process::exit(1);
                }
            })?;

        for path in &cli.watch {
            watcher.watch(path, RecursiveMode::Recursive)?;
        }

        loop {
            sleep(Duration::from_secs_f32(0.25));
        }
    } else {
        // Watch

        // Error if more than one command
        if cli.arguments.len() > 1 {
            eprintln!("ERROR: Watch mode only works with a single command!");
            std::process::exit(1);
        }

        // Run the command in a child process
        let command = Command::new(&cli.arguments[0]);
        let (mut process, mut ts) = run(&shell, &command);

        // Get watched directories & files
        let (dirs, mut hashes) = watched(&cli.watch);
        let ignored = Ignore::default();
        let pwd = std::env::current_dir().unwrap();

        let debounce = std::time::Duration::from_secs_f32(cli.debounce);

        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    let now = std::time::Instant::now();
                    match event.kind {
                        EventKind::Create(_) | EventKind::Remove(_) => {
                            // Created or deleted a file/directory
                            for path in event
                                .paths
                                .iter()
                                .map(|x| x.strip_prefix(&pwd).unwrap().to_path_buf())
                                .filter(|x| not_ignored(x, &ignored, &dirs, &hashes))
                            {
                                // In a watched directory...

                                if now - ts > debounce {
                                    // Kill the command (if still running)
                                    if let Ok(None) = process.try_wait() {
                                        process.kill().expect("kill process");
                                    }
                                    shell.print_fence(2);

                                    println!(
                                        "* {}: `{}`\n",
                                        match event.kind {
                                            EventKind::Create(_) => "Created",
                                            EventKind::Remove(_) => "Removed",
                                            _ => unreachable!(),
                                        },
                                        path.display(),
                                    );

                                    // Run the command again
                                    (process, ts) = run(&shell, &command);

                                    break;
                                }
                            }
                        }
                        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                            // Wrote a file
                            let mut not_restarted = true;
                            let paths = event
                                .paths
                                .iter()
                                .map(|x| x.strip_prefix(&pwd).unwrap().to_path_buf())
                                .filter(|x| not_ignored(x, &ignored, &dirs, &hashes))
                                .collect::<Vec<_>>();
                            for path in paths {
                                if let Some(h1) = hashes.get(&path) {
                                    let h2 = hash(&path);
                                    if h2 != *h1 {
                                        // File changed...

                                        // Update the hash
                                        hashes.insert(path.clone(), h2);

                                        if not_restarted && now - ts > debounce {
                                            // Kill the command (if still running)
                                            if let Ok(None) = process.try_wait() {
                                                process.kill().expect("kill process");
                                            }
                                            shell.print_fence(2);

                                            println!("* Modified: `{}`\n", path.display());

                                            // Run the command again
                                            (process, ts) = run(&shell, &command);
                                            not_restarted = false;
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(_e) => {
                    std::process::exit(1);
                }
            })?;

        for path in &cli.watch {
            watcher.watch(path, RecursiveMode::Recursive)?;
        }

        loop {
            sleep(Duration::from_secs_f32(0.25));
        }
    }

    Ok(())
}

fn run(shell: &Shell, command: &Command) -> (std::process::Child, std::time::Instant) {
    shell.interactive_prompt(false);
    println!("{}", command.command);
    shell.interactive_prompt_reset();
    (shell.run1_async(command), std::time::Instant::now())
}

fn watched(args: &[PathBuf]) -> (Vec<PathBuf>, BTreeMap<PathBuf, String>) {
    // Get directories
    let dirs = args
        .iter()
        .filter(|x| x.is_dir())
        .cloned()
        .collect::<Vec<_>>();

    // Get hashes for all watched files
    let hashes = args
        .iter()
        .filter(|x| x.is_file())
        .cloned()
        .chain(dirs.iter().flat_map(|x| {
            ignore::Walk::new(x)
                .flatten()
                .filter(|x| x.path().is_file())
                .map(|x| {
                    let path = x.into_path();
                    match path.strip_prefix("./") {
                        Ok(p) => p.to_path_buf(),
                        Err(_e) => path,
                    }
                })
        }))
        .map(|x| {
            let h = hash(&x);
            (x, h)
        })
        .collect::<BTreeMap<_, _>>();

    (dirs, hashes)
}

fn not_ignored(
    path: &Path,
    ignored: &Ignore,
    dirs: &[PathBuf],
    hashes: &BTreeMap<PathBuf, String>,
) -> bool {
    let path = path.to_owned();
    !ignored.check(&path) && !dirs.contains(&path) && !hashes.contains_key(&path)
}

fn hash(path: &Path) -> String {
    fhc::file_blake3(path).unwrap().remove(0).1
}
