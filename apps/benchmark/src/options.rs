//! Command-line parsing. The runner is headless; there is no window option.

use std::path::PathBuf;

pub const USAGE: &str = "\
Usage: acme-benchmark [--quick] [--json PATH] [--help]

  --quick       small warm-up and sample counts (smoke run, noisy numbers)
  --json PATH   also write machine-readable results (schema v1) to PATH
  --help        show this message

The benchmark is headless: no window and no GPU device are created.
";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Options {
    pub quick: bool,
    pub json: Option<PathBuf>,
    pub help: bool,
}

/// Parse arguments (without the program name).
pub fn parse<I>(args: I) -> Result<Options, String>
where
    I: IntoIterator<Item = String>,
{
    let mut options = Options::default();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--quick" => options.quick = true,
            "--json" => {
                let path = args.next().ok_or("--json needs a PATH")?;
                options.json = Some(non_empty_path(&path)?);
            }
            "--help" | "-h" => options.help = true,
            "--window" => {
                return Err("--window was removed: the benchmark runs headless only".into());
            }
            other => match other.strip_prefix("--json=") {
                Some(path) => options.json = Some(non_empty_path(path)?),
                None => return Err(format!("unknown argument `{other}`")),
            },
        }
    }
    Ok(options)
}

fn non_empty_path(path: &str) -> Result<PathBuf, String> {
    if path.is_empty() {
        return Err("--json PATH must not be empty".into());
    }
    if path.starts_with("--") {
        return Err(format!("--json PATH looks like an option: `{path}`"));
    }
    Ok(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_arguments_means_full_headless_run() {
        assert_eq!(parse(args(&[])), Ok(Options::default()));
    }

    #[test]
    fn quick_and_json_path_are_parsed() {
        let parsed = parse(args(&["--quick", "--json", "out/bench.json"])).expect("valid");
        assert!(parsed.quick);
        assert_eq!(parsed.json, Some(PathBuf::from("out/bench.json")));
    }

    #[test]
    fn json_equals_form_is_accepted() {
        let parsed = parse(args(&["--json=a.json"])).expect("valid");
        assert_eq!(parsed.json, Some(PathBuf::from("a.json")));
    }

    #[test]
    fn window_flag_is_rejected_with_reason() {
        let error = parse(args(&["--window"])).expect_err("removed flag");
        assert!(error.contains("headless"));
    }

    #[test]
    fn missing_or_empty_json_path_is_rejected() {
        assert!(parse(args(&["--json"])).is_err());
        assert!(parse(args(&["--json="])).is_err());
    }

    #[test]
    fn option_in_place_of_json_path_is_rejected() {
        assert!(parse(args(&["--json", "--quick"])).is_err());
    }

    #[test]
    fn unknown_argument_is_rejected() {
        assert!(parse(args(&["--frobnicate"])).is_err());
    }
}
