use clap::{App, Arg};
use std::fmt;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};
use colored::*;

pub struct Git {
    clean: bool,
    nosync: bool,
}

pub enum ProjectKind {
    NodeJS { installed: bool, lockfile: bool },
    Rust { installed: bool },
}

pub struct Project {
    git: Option<Git>,
    kind: Option<ProjectKind>,
}

impl Project {
    fn is_project(&self) -> bool {
        self.git.is_some() || self.kind.is_some()
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let &Some(Git { clean, nosync }) = &self.git {
            write!(
                f,
                "  found {} with {} worktree and {}",
                "Git".bold(),
                if clean { "clean" } else { "dirty" },
                if nosync { "need sync" } else { "synced" }
            )?;
        } else {
            write!(f, "no {} found", "Git".bold(),)?;
        }
        if let Some(kind) = &self.kind {
            writeln!(f)?;
            match kind {
                &ProjectKind::NodeJS {
                    installed,
                    lockfile,
                } => write!(
                    f,
                    "  found {} {} ({} lockfile)",
                    if installed {
                        "installed"
                    } else {
                        "uninitialized"
                    },
                    "Node.js".bold(),
                    if lockfile { "has" } else { "no" }
                )?,
                &ProjectKind::Rust { installed } => write!(
                    f,
                    "found {} Rust",
                    if installed {
                        "installed"
                    } else {
                        "uninitialized"
                    },
                )?,
            }
        }
        Ok(())
    }
}

fn is_file(path_buf: &mut PathBuf, file_name: &str) -> bool {
    path_buf.push(file_name);
    let result = path_buf.is_file();
    path_buf.pop();
    result
}

fn is_dir(path_buf: &mut PathBuf, file_name: &str) -> bool {
    path_buf.push(file_name);
    let result = path_buf.is_dir();
    path_buf.pop();
    result
}

fn examine(directory: &Path) -> Project {
    let mut path_buf = directory.to_path_buf();
    // Check if the folder has a .git folder.
    path_buf.push(".git");
    let git = if path_buf.is_dir() {
        Some(Git {
            clean: true,
            nosync: true,
        })
    } else {
        None
    };
    path_buf.pop();
    let has_package_json = is_file(&mut path_buf, "package.json");
    let has_lockfile = is_file(&mut path_buf, "package-lock.json") || is_file(&mut path_buf, "yarn.lock");
    let has_node_modules = is_dir(&mut path_buf, "node_modules");
    let kind = if has_package_json || has_lockfile || has_node_modules {
        Some(ProjectKind::NodeJS {
            installed: has_node_modules,
            lockfile: has_lockfile,
        })
    } else {
        None
    };
    path_buf.pop();
    // Ending
    Project { git, kind }
}

fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        let project = examine(dir);
        if project.is_project() {
            let dir_path = dir.display().to_string().green();
            println!("[{}]", dir_path);
            println!("{}", project);
        } else {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                } else {
                    cb(&entry);
                }
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum AppError {
    IOError(io::Error),
    ArgNotFoundError,
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        AppError::IOError(error)
    }
}

fn main() -> Result<(), AppError> {
    let matches = App::new("Project Finder")
        .version("0.0.1")
        .author("Luyu Cheng <luyu@hey.com>")
        .about("Find all projects in your deeply nested development directory.")
        .arg(
            Arg::new("INPUT")
                .about("Sets the input directory to scan.")
                .required(true)
                .index(1),
        )
        .get_matches();
    let input_directory = matches
        .value_of("INPUT")
        .ok_or(AppError::ArgNotFoundError)?;
    visit_dirs(Path::new(input_directory), &|_| {
        // println!("{:?}", entry.file_name());
    })?;
    Ok(())
}
