/// Helper binary: generates shell completions and man pages from the CLI definition.
///
/// Usage:
///   scale-bridge-generate <output-dir>
///
/// Writes:
///   <output-dir>/completions/scale-bridge.bash
///   <output-dir>/completions/scale-bridge.zsh
///   <output-dir>/completions/scale-bridge.fish
///   <output-dir>/man/scale-bridge.1
mod args;

use args::Cli;
use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use clap_mangen::Man;
use std::path::PathBuf;

fn main() {
    let out_dir: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    let completions_dir = out_dir.join("completions");
    let man_dir = out_dir.join("man");

    std::fs::create_dir_all(&completions_dir)
        .unwrap_or_else(|e| panic!("cannot create {}: {e}", completions_dir.display()));
    std::fs::create_dir_all(&man_dir)
        .unwrap_or_else(|e| panic!("cannot create {}: {e}", man_dir.display()));

    let mut cmd = Cli::command();

    // Shell completions
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        let path = generate_to(shell, &mut cmd, "scale-bridge", &completions_dir)
            .unwrap_or_else(|e| panic!("completion generation failed for {shell}: {e}"));
        println!("wrote {}", path.display());
    }

    // Man page
    let man = Man::new(cmd);
    let man_path = man_dir.join("scale-bridge.1");
    let mut buf = Vec::new();
    man.render(&mut buf)
        .unwrap_or_else(|e| panic!("man page render failed: {e}"));
    std::fs::write(&man_path, buf)
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", man_path.display()));
    println!("wrote {}", man_path.display());
}
