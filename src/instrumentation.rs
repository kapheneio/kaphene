use color_eyre::eyre::WrapErr;
use std::{error::Error, io::IsTerminal};
use tracing::Subscriber;
use tracing_subscriber::{
    filter::Directive,
    layer::{Layer, SubscriberExt},
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter,
};
use crate::logger::Logger;

#[derive(clap::Args, Debug, Default)]
pub struct Instrumentation {
    /// Enable debug logs, -vv for trace
    #[clap(
        short = 'v',
        env = "KAPHENE_LOG_VERBOSITY",
        long, action = clap::ArgAction::Count,
        global = true
    )]
    pub verbose: u8,
    /// Which logger to use
    #[clap(
        long,
        env = "KAPHENE_LOGGER",
        default_value_t = Default::default(),
        global = true
    )]
    pub(crate) logger: Logger,
    /// Tracing directives
    #[clap(
        long = "log-directive",
        global = true,
        env = "KAPHENE_LOG_DIRECTIVES",
        value_delimiter = ',', num_args = 0..
    )]
    pub(crate) log_directives: Vec<Directive>,
}

impl Instrumentation {
    pub fn log_level(&self) -> String {
        match self.verbose {
            0 => "info",
            1 => "debug",
            _ => "trace",
        }.to_string()
    }

    pub fn init(&self) -> color_eyre::Result<()> {
        let filter = self.filter()?;

        let registry = tracing_subscriber::registry()
            .with(filter)
            .with(tracing_error::ErrorLayer::default());

        match self.logger {
            Logger::Json => registry.with(self.fmt_layer_json()).try_init()?,
            Logger::Pretty => registry.with(self.fmt_layer_pretty()).try_init()?,
            Logger::Terse => registry.with(self.fmt_layer_terse()).try_init()?,
            Logger::Verbose => registry.with(self.fmt_layer_verbose()).try_init()?,
        }

        Ok(())
    }

    pub fn filter(&self) -> color_eyre::Result<EnvFilter> {
        let mut layer = match EnvFilter::try_from_default_env() {
            Ok(l) => l,
            Err(e) => {
                // Attempt to recover from missing envar.
                if let Some(source) = e.source() {
                    match source.downcast_ref::<std::env::VarError>() {
                        Some(std::env::VarError::NotPresent) => (),
                        _ => return Err(e).wrap_err_with(|| "parsing RUST_LOG directives"),
                    }
                }

                if self.log_directives.is_empty() {
                    EnvFilter::try_new(&format!(
                        "{}={}",
                        env!("CARGO_PKG_NAME").replace("-", "_"),
                        self.log_level()
                    ))?
                } else {
                    EnvFilter::try_new("")?
                }
            }
        };

        for directive in &self.log_directives {
            layer = layer.add_directive(directive.clone());
        }

        Ok(layer)
    }

    pub fn fmt_layer_json<S>(&self) -> impl Layer<S>
        where S: Subscriber + for<'span> LookupSpan<'span>, {
        tracing_subscriber::fmt::Layer::new()
            .with_ansi(std::io::stderr().is_terminal())
            .with_writer(std::io::stderr)
            .json()
    }

    pub fn fmt_layer_pretty<S>(&self) -> impl Layer<S>
        where S: Subscriber + for<'span> LookupSpan<'span>, {
        tracing_subscriber::fmt::Layer::new()
            .with_ansi(std::io::stderr().is_terminal())
            .with_writer(std::io::stderr)
            .pretty()
    }

    pub fn fmt_layer_terse<S>(&self) -> impl Layer<S>
        where S: Subscriber + for<'span> LookupSpan<'span>, {
        tracing_subscriber::fmt::Layer::new()
            .with_ansi(std::io::stderr().is_terminal())
            .with_writer(std::io::stderr)
            .compact()
            .without_time()
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
    }

    pub fn fmt_layer_verbose<S>(&self) -> impl Layer<S>
        where S: Subscriber + for<'span> LookupSpan<'span>, {
        tracing_subscriber::fmt::Layer::new()
            .with_ansi(std::io::stderr().is_terminal())
            .with_writer(std::io::stderr)
    }
}