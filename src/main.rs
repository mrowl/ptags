use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    tag: String,
    #[structopt(parse(from_os_str))]
    path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    let mut emitted_paths: HashSet<PathBuf> = HashSet::new();
    let mut q: VecDeque<PathBuf> = VecDeque::new();

    let source_contents = std::fs::read_to_string(&args.path).expect("could not read source file");
    let mut path_arg = args.path;
    if !path_arg.is_dir() {
        path_arg.pop();
    }
    q.push_back(path_arg);

    while !q.is_empty() {
        let mut path: PathBuf = q.pop_front().unwrap();
        if !emitted_paths.contains(&path) && path.is_dir() {
            println!("{}", path.display());
            emitted_paths.insert(path.to_owned());
            push_deps(&mut path, &mut q);
        }
    }

    //println!("file_content: {}", source_contents);
    Ok(())
}

fn push_deps(path: &mut PathBuf, mut q: &mut VecDeque<PathBuf>) -> Result<bool, String> {
    path.push("BUILD");
    let result = std::fs::read_to_string(&path);
    match result {
        Ok(content) => {
            content
                .lines()
                .skip_while(|line| !line.contains("dependencies = ["))
                .skip(1)
                .take_while(|line| !line.contains("],"))
                .flat_map(|line| {
                    line.trim()
                        .trim_matches(|c| c == '"' || c == ',')
                        .split(':')
                        .nth(0)
                })
                .map(|line| PathBuf::from(line))
                .for_each(|p| q.push_back(p));
            Ok(true)
        }
        Err(_error) => {
            path.pop();
            match path.parent() {
                Some(p) => push_deps(&mut p.to_path_buf(), &mut q),
                None => Err(String::from(
                    "Reached root directory and could not find a BUILD file",
                )),
            }
        }
    }
}
