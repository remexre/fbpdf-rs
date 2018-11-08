use failure::{Error, Fallible};
use std::path::PathBuf;

#[derive(structopt::StructOpt)]
#[structopt(raw(setting = "::structopt::clap::AppSettings::ColoredHelp"))]
pub struct Options {
    /// Turns off message output.
    #[structopt(short = "q", long = "quiet")]
    pub quiet: bool,

    /// Increases the verbosity.
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    pub verbose: usize,

    /// The file to read.
    #[structopt(parse(from_os_str))]
    pub file: PathBuf,

    /// The password for the PDF, if any.
    #[structopt(short = "p", long = "password", default_value = "")]
    pub password: String,
}

impl Options {
    /// Sets up logging as specified by the `-q` and `-v` flags.
    pub fn start_logger(&self) -> Fallible<()> {
        stderrlog::new()
            .quiet(self.quiet)
            .verbosity(self.verbose)
            .init()
            .map_err(Error::from)
    }
}
