use std::path::PathBuf;

use clap::Parser;
use glob::Pattern;
use ts_vis_rs::{Scanner, ScannerOptions};

#[derive(Parser, Debug)]
struct CliArgs {
    pub entry: PathBuf,

    #[arg(short, long)]
    pub cwd: Option<PathBuf>,

    #[arg(short, long, default_values_t = vec!["*node_modules*".to_string()])]
    pub filter: Vec<String>,

    #[arg(short, long)]
    pub include: Option<Vec<String>>,
}

fn main() {
    let args = CliArgs::parse();

    let scanner = Scanner::new(
        args.cwd
            .unwrap_or(find_root(&args.entry).expect("Could not find root directory")),
        ScannerOptions {
            filter: args
                .filter
                .iter()
                .map(|f| Pattern::new(f).unwrap())
                .collect(),
            include: args.include.map(|i| i.iter().map(|p| Pattern::new(p).unwrap()).collect())
        },
    );
    let graph = scanner.scan(&args.entry);

    println!("{}", graph.print_graphviz());
}

fn find_root(entry: &PathBuf) -> Option<PathBuf> {
    match entry.join("package.json").exists() {
        true => Some(entry.to_path_buf()),
        false => find_root(
            &entry
                .parent()
                .expect("Could not find root directory")
                .to_path_buf(),
        ),
    }
}
