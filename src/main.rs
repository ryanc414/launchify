use regex::Regex;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

fn main() {
    let args = Cli::from_args();
    println!("program: {}, period: {:?}", args.program, args.period);

    match run(args) {
        Ok(_) => println!("Completely successfully"),
        Err(err) => {
            println!("Error: {}", err);
            std::process::exit(1);
        }
    }
}

fn run(args: Cli) -> io::Result<()> {
    let cfg = LaunchConfig::from_cli(args)?;
    cfg.ensure_log_dirs()?;
    let plist = cfg.to_plist();
    println!("got program plist:\n{}", plist);
    Ok(())
}

#[derive(Debug)]
enum Period {
    Day(u64),
    Hour(u64),
    Minute(u64),
    Second(u64),
}

impl Period {
    fn to_seconds(&self) -> u64 {
        match self {
            Self::Day(days) => 24 * 60 * 60 * days,
            Self::Hour(hours) => 60 * 60 * hours,
            Self::Minute(mins) => 60 * mins,
            &Self::Second(secs) => secs,
        }
    }
}

#[derive(Debug, Clone)]
struct ParsePeriodError;

impl fmt::Display for ParsePeriodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error parsing period")
    }
}

impl FromStr for Period {
    type Err = ParsePeriodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^(\d+)(d|h|m|s)$").unwrap();
        let caps = match re.captures(s) {
            Some(caps) => caps,
            None => return Err(ParsePeriodError),
        };

        if caps.len() != 3 {
            return Err(ParsePeriodError);
        }

        let value: u64 = match caps[1].parse() {
            Ok(val) => val,
            Err(_) => return Err(ParsePeriodError),
        };

        let period = match &caps[2] {
            "d" => Period::Day,
            "h" => Period::Hour,
            "m" => Period::Minute,
            "s" => Period::Second,
            _ => return Err(ParsePeriodError),
        };

        Ok(period(value))
    }
}

#[derive(StructOpt)]
struct Cli {
    program: String,
    period: Period,
}

struct LaunchConfig {
    name: String,
    program_path: PathBuf,
    start_interval: u64,
    log_dir: PathBuf,
}

impl LaunchConfig {
    fn from_cli(args: Cli) -> io::Result<Self> {
        let path = fs::canonicalize(args.program)?;
        let file_stem = match path.file_stem() {
            Some(stem) => stem,
            None => return Err(io::Error::new(io::ErrorKind::Other, "no file name")),
        };
        let file_stem = match file_stem.to_str() {
            Some(stem) => stem.to_owned(),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "could not convert filename to string",
                ))
            }
        };

        let start_interval = args.period.to_seconds();
        let log_dir = get_log_dir(&file_stem)?;

        Ok(Self {
            name: file_stem,
            program_path: path,
            start_interval,
            log_dir,
        })
    }

    fn ensure_log_dirs(&self) -> io::Result<()> {
        fs::create_dir_all(&self.log_dir)
    }

    fn to_plist(&self) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\"
  \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
    <key>Label</key>
    <string>{}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>StartInterval</key>
    <integer>{}</integer>
    <key>StandardOutPath</key>
    <string>{}</string>
    <key>StandardErrorPath</key>
    <string>{}</string>
</dict>
</plist>",
            self.name,
            self.program_path.to_str().unwrap(),
            self.start_interval,
            self.stdout_path().to_str().unwrap(),
            self.stderr_path().to_str().unwrap(),
        )
    }

    fn stdout_path(&self) -> PathBuf {
        let mut path = self.log_dir.to_owned();
        path.push("stdout");
        path
    }

    fn stderr_path(&self) -> PathBuf {
        let mut path = self.log_dir.to_owned();
        path.push("stderr");
        path
    }
}

fn get_log_dir(name: &str) -> io::Result<PathBuf> {
    let mut log_dir = match dirs::home_dir() {
        Some(home) => home,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not find user home directory",
            ));
        }
    };
    log_dir.push("logs");
    log_dir.push(name);

    Ok(log_dir)
}
