use std::error::Error;
use std::env;
use std::fmt;
use std::io::{Error as IOError, Result as IOResult, Write};
use std::io::{stdout, stderr};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use argparse::{
    ArgumentParser,
    Collect,
    FromCommandLine,
    Store,
};
use libc::pid_t;

use super::syscall::nsenter;

#[derive(Clone, Debug)]
pub struct Volume {
    source: PathBuf,
    destination: PathBuf,
    readonly: bool,
}

#[derive(Clone, Debug)]
pub struct NSExecutor {
    pid: pid_t,
    volumes: Vec<Volume>,
    command_args: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct VolumeParseError {
    value: String
}

pub trait FromArguments<T>: Sized {
    fn from_arguments(args: Vec<String>, stdout: &mut Write, stderr: &mut Write) -> Result<T, i32>;

    fn from_cmd_arguments() -> Result<T, i32> {
        Self::from_arguments(env::args().collect(), &mut stdout(), &mut stderr())
    }
}

impl Volume {
    pub fn source(&self) -> &PathBuf {
        &self.source
    }

    pub fn destination(&self) -> &PathBuf {
        &self.destination
    }
}

impl NSExecutor {
    pub fn volumes(&self) -> &Vec<Volume> {
        &self.volumes
    }

    pub fn nsenter(&self) -> IOResult<()> {
        nsenter(&self.pid)
    }

    pub fn exec(&self) -> IOError {
        Command::new(&self.command_args[0]).args(&self.command_args[1..]).exec()
    }
}

impl FromArguments<NSExecutor> for NSExecutor {
    fn from_arguments(args: Vec<String>, stdout: &mut Write, stderr: &mut Write) -> Result<NSExecutor, i32> {
        let mut executor = NSExecutor{
            pid: 0,
            volumes: Vec::new(),
            command_args: Vec::new(),
        };

        try!({
            let mut parser = ArgumentParser::new();
            parser.refer(&mut executor.pid)
                .add_option(
                    &["-t", "--target"],
                    Store,
                    "target process to get namespaces from",
                )
                .required();
            parser.refer(&mut executor.volumes)
                .add_option(
                    &["-v", "--volume"],
                    Collect,
                    "bind mount a volume",
                );
            parser.refer(&mut executor.command_args)
                .add_argument(
                    "arg",
                    Collect,
                    "command line args to run",
                );
            parser.parse(args, stdout, stderr)
        });

        if executor.command_args.is_empty() {
            executor.command_args.push("sh".to_string());
        }
        Ok(executor)
    }
}

impl FromStr for Volume {
    type Err = VolumeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split(':');
        match iter.next() {
            Some(source) => match iter.next() {
                Some(destination) => match iter.next() {
                    Some(access @ "ro") | Some (access @ "rw") => match iter.next() {
                        None => Ok(Volume {
                            source: PathBuf::from(source),
                            destination: PathBuf::from(destination),
                            readonly: access == "ro",
                        }),
                        Some(_) => Err(VolumeParseError { value: s.to_string() }), 
                    },
                    None => Ok(Volume {
                        source: PathBuf::from(source),
                        destination: PathBuf::from(destination),
                        readonly: false,
                    }),
                    Some(_) => Err(VolumeParseError { value: s.to_string() }),
                },
                None => Ok(Volume {
                    source: PathBuf::from(source),
                    destination: PathBuf::from(source),
                    readonly: false,
                }),
            },
            None => Err(VolumeParseError { value: s.to_string() }),
        }
    }
}

impl FromCommandLine for Volume {
    fn from_argument(arg: &str) -> Result<Self, String> {
        FromStr::from_str(arg).map_err(|e| format!("{:?}", e))
    }
}

impl Error for VolumeParseError {
    fn description(&self) -> &str {
        "invalid volume spec ([src:]dest[:rw|:ro])"
    }
}

impl fmt::Display for VolumeParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}: {:?}", self.description(), self.value)
    }
}

