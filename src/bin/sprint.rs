use anyhow::Result;
use clap::Parser;
use sprint::*;

#[derive(Parser)]
#[command(about, version, max_term_width = 80)]
struct Cli {
    /// Shell
    #[arg(short, value_name = "STRING", default_value = "sh -c")]
    shell: String,

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
    } else {
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
    }

    Ok(())
}
