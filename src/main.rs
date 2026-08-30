mod curly_expand;
use clap::{Parser, ValueEnum};
use std::io::{self, IsTerminal, Read};
use std::process::ExitCode;
use turnsignal::{Observation, ObservationKind, SignalMode, Turnsignal};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Agent output or intended action. Reads stdin when omitted.
    text: Option<String>,

    /// Treat the input as an intended action and anticipate its consequences.
    #[arg(long)]
    projected: bool,

    /// How the signal is delivered.
    #[arg(long, value_enum, default_value_t = Mode::Advisory)]
    mode: Mode,

    /// State immediately before this observation.
    #[arg(long)]
    state: Option<String>,

    /// Name of the agent or tool that produced the observation.
    #[arg(long, default_value = "agent")]
    source: String,

    /// Emit the full canonical delivery as JSON.
    #[arg(long)]
    json: bool,

    /// Remove insignificant whitespace from JSON output.
    #[arg(long, requires = "json")]
    compact: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum Mode {
    #[default]
    Advisory,
    Injective,
    Routing,
}

impl From<Mode> for SignalMode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Advisory => Self::Advisory,
            Mode::Injective => Self::Injective,
            Mode::Routing => Self::Routing,
        }
    }
}

fn __curly_original_main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("turnsignal: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let content = match cli.text {
        Some(text) if !text.trim().is_empty() => text,
        Some(_) => return Err("observation cannot be empty".into()),
        None if io::stdin().is_terminal() => {
            return Err("provide an observation argument or pipe one on stdin".into());
        }
        None => {
            let mut content = String::new();
            io::stdin()
                .read_to_string(&mut content)
                .map_err(|error| format!("could not read stdin: {error}"))?;
            if content.trim().is_empty() {
                return Err("observation cannot be empty".into());
            }
            content
        }
    };

    let observation = Observation {
        source: cli.source,
        kind: if cli.projected {
            ObservationKind::Projected
        } else {
            ObservationKind::Result
        },
        content,
        metadata: Default::default(),
    };
    let delivery =
        Turnsignal::default().dispatch(&observation, cli.state.as_deref(), cli.mode.into());

    if cli.json {
        let output = if cli.compact {
            serde_json::to_string(&delivery)
        } else {
            serde_json::to_string_pretty(&delivery)
        }
        .map_err(|error| format!("could not serialize signal: {error}"))?;
        println!("{output}");
    } else {
        println!("{}", delivery.message);
    }

    Ok(())
}

fn main() {
    let raw_args: Vec<String> = std::env::args().collect();
    let mut positions: Vec<usize> = Vec::new();
    let mut fields: Vec<Vec<String>> = Vec::new();
    for (__i, __a) in raw_args.iter().enumerate() {
        if __a == "--state" {
            if let Some(__v) = raw_args.get(__i + 1) {
                positions.push(__i + 1);
                fields.push(curly_expand::expand_or_literal(__v));
            }
            break;
        } else if let Some(__v) = __a.strip_prefix("--state=") {
            positions.push(__i);
            fields.push(
                curly_expand::expand_or_literal(__v)
                    .into_iter()
                    .map(|v| format!("--state={}", v))
                    .collect(),
            );
            break;
        }
    }
    for (__i, __a) in raw_args.iter().enumerate() {
        if __a == "--source" {
            if let Some(__v) = raw_args.get(__i + 1) {
                positions.push(__i + 1);
                fields.push(curly_expand::expand_or_literal(__v));
            }
            break;
        } else if let Some(__v) = __a.strip_prefix("--source=") {
            positions.push(__i);
            fields.push(
                curly_expand::expand_or_literal(__v)
                    .into_iter()
                    .map(|v| format!("--source={}", v))
                    .collect(),
            );
            break;
        }
    }

    if fields.is_empty() || fields.iter().all(|f| f.len() <= 1) {
        __curly_original_main();
        return;
    }

    let combos = curly_expand::cartesian(&fields);
    let exe = std::env::current_exe().expect("resolve current exe");
    let mut had_failure = false;
    for combo in &combos {
        let mut new_args = raw_args.clone();
        for (slot, value) in positions.iter().zip(combo.iter()) {
            new_args[*slot] = value.clone();
        }
        let status = std::process::Command::new(&exe)
            .args(&new_args[1..])
            .status()
            .expect("failed to re-exec self");
        if !status.success() {
            had_failure = true;
        }
    }
    if had_failure {
        std::process::exit(1);
    }
}
