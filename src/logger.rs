use clap;

#[derive(Clone, Default, Debug, clap::ValueEnum)]
pub(crate) enum Logger {
    Json,
    Pretty,
    #[default]
    Terse,
    Verbose,
}

impl std::fmt::Display for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let logger = match self {
            Logger::Json => "json",
            Logger::Pretty => "pretty",
            Logger::Terse => "terse",
            Logger::Verbose => "verbose",
        };

        write!(f, "{}", logger)
    }
}