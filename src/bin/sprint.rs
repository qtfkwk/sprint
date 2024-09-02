use {
    anyhow::Result,
    clap::Parser,
    glob::glob,
    notify::{
        event::{AccessKind, AccessMode},
        Event, EventKind, RecursiveMode, Watcher,
    },
    sprint::*,
    std::{collections::BTreeMap, path::PathBuf, thread::sleep, time::Duration},
};

#[derive(Parser)]
#[command(about, version, max_term_width = 80)]
struct Cli {
    /// Shell
    #[arg(short, value_name = "STRING", default_value = "sh -c")]
    shell: String,

    /// Watch files/directories and rerun command on change; see also `-d` option
    #[arg(short, value_name = "PATH")]
    watch: Vec<PathBuf>,

    /// Debounce; used only with `-w`
    #[arg(short, value_name = "SECONDS", default_value = "5.0")]
    debounce: f32,

    /// Fence
    #[arg(short, value_name = "STRING", default_value = "```")]
    fence: String,

    /// Info
    #[arg(short, value_name = "STRING", default_value = "text")]
    info: String,

    /// Prompt
    #[arg(short, value_name = "STRING", default_value = "$ ")]
    prompt: String,

    /// Force enable/disable terminal colors
    #[arg(long, default_value = "auto")]
    color: ColorOverride,

    /// File(s) or command(s)
    #[arg(value_name = "STRING")]
    arguments: Vec<String>,
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

    if cli.arguments.is_empty() {
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
    } else if cli.watch.is_empty() {
        // Run given commands / files

        let results = shell.run(
            &cli.arguments
                .iter()
                .map(|x| Command::new(x))
                .collect::<Vec<_>>(),
        );

        // Exit with the code of the last command
        std::process::exit(if let Some(code) = results.last().unwrap().code {
            code
        } else {
            1
        });
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

        // Get canonical directories
        let dirs = cli
            .watch
            .iter()
            .filter_map(|x| {
                if x.is_dir() {
                    Some(x.canonicalize().unwrap())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Glob files in watched directories
        let globbed = find_files(&dirs);

        // Get hashes for all watched files
        let mut hashes = cli
            .watch
            .iter()
            .filter(|x| x.is_file())
            .chain(globbed.iter())
            .map(|x| (x.canonicalize().unwrap(), fhc::file_blake3(x).unwrap()))
            .collect::<BTreeMap<_, _>>();

        let debounce = std::time::Duration::from_secs_f32(cli.debounce);

        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<Event>| match res {
                Ok(event) => {
                    match event.kind {
                        EventKind::Create(_) | EventKind::Remove(_) => {
                            // Created or deleted a file/directory
                            for path in event.paths {
                                for dir in &dirs {
                                    if path.starts_with(dir) {
                                        // In a watched directory...

                                        if std::time::Instant::now() - ts > debounce {
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
                                                path.display()
                                            );

                                            // Run the command again
                                            (process, ts) = run(&shell, &command);

                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                            // Wrote a file
                            let mut not_restarted = true;
                            for path in event.paths {
                                if let Some(h1) = hashes.get(&path) {
                                    let h2 = fhc::file_blake3(&path).unwrap();
                                    if h2 != *h1 {
                                        // File changed...

                                        // Update the hash
                                        hashes.insert(path.clone(), h2);

                                        if not_restarted
                                            && std::time::Instant::now() - ts > debounce
                                        {
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
                    // println!("watch error: {:?}", e);
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

fn find_files(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let original_directory = std::env::current_dir().unwrap();
    let mut r = vec![];
    for dir in dirs {
        std::env::set_current_dir(dir).unwrap();
        for path in glob("**/*").unwrap().flatten() {
            if path.is_file() {
                r.push(path.canonicalize().unwrap());
            }
        }
        std::env::set_current_dir(&original_directory).unwrap();
    }
    r
}

fn run(shell: &Shell, command: &Command) -> (std::process::Child, std::time::Instant) {
    shell.interactive_prompt(false);
    println!("{}", command.command);
    shell.interactive_prompt_reset();
    (shell.run1_async(command), std::time::Instant::now())
}
