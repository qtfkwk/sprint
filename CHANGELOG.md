# Changelog

* 0.1.0 (2023-12-22): Initial release
    * 0.1.1 (2023-12-24): Fix readme
    * 0.1.2 (2023-12-24): Fix readme
* 0.2.0 (2023-12-26): Redesign; update dependencies
* 0.3.0 (2023-12-27): Add error handling
* 0.4.0 (2023-12-29): Fix error handling
* 0.5.0 (2024-01-05): Add CLI; update dependencies
* 0.6.0 (2024-01-05): Fix script mode output
    * 0.6.1 (2024-07-26): Fix makefile; update dependencies
* 0.7.0 (2024-08-04): Switch terminal colors from [`bunt`] to [`owo-colors`] ([ref][rain-rust-cli-colors]); add `--color` option; update dependencies
    * 0.7.1 (2024-08-04): Fix color init
    * 0.7.2 (2024-08-16): Update dependencies
    * 0.7.3 (2024-08-22): Fix readme; add `commit` target to makefile; update dependencies
* 0.8.0 (2024-09-02): Add watch mode / `-w` and `-d` options; streamline docstrings; add the print_fence and run1_async methods; update dependencies
* 0.9.0 (2024-09-03): Make watch mode respect `.gitignore` file and enable running without a command; add long options; fix makefile
* 0.10.0 (2024-09-04): Use [`ignore-check`] crate to check paths from notify events; update dependencies
    * 0.10.1 (2024-09-04): Fix changelog
    * 0.10.2 (2024-09-04): Fix bug introduced in 0.8.0 that made any command with stdin attempt to write to stdin twice
* 0.11.0 (2024-10-24): Add clap color; switch from owo-colors' support-colors feature to [`anstream`]; update dependencies
    * 0.11.1 (2024-12-04): Update dependencies
    * 0.11.2 (2024-12-04): Update dependencies
    * 0.11.3 (2024-12-19): Fix starting fence for custom shells; update dependencies
    * 0.11.4 (2025-02-21): Update dependencies
    * 0.11.5 (2025-04-16): Update dependencies
* 0.12.0 (2025-08-28): Update dependencies; 2024 edition
    * 0.12.1 (2025-10-27): Update dependencies
    * 0.12.2 (2025-11-11): Use [`clap-cargo`] `CLAP_STYLING`; update dependencies
    * 0.12.3 (2025-11-13): Update dependencies; clippy fixes

[`anstream`]: https://crates.io/crates/anstream
[`bunt`]: https://crates.io/crates/bunt
[`clap-cargo`]: https://crates.io/crates/clap-cargo
[`ignore-check`]: https://crates.io/crates/ignore-check
[`owo-colors`]: https://crates.io/crates/owo-colors
[rain-rust-cli-colors]: https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html

