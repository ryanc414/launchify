use handlebars::Handlebars;
use regex::Regex;
use serde_json::json;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use structopt::StructOpt;
use which::which;

const PLIST_TEMPLATE: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\"
  \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<dict>
    <key>Label</key>
    <string>{{name}}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{{program_path}}</string>{{#each args}}
        <string>{{this}}</string>{{/each}}
    </array>
    <key>StartInterval</key>
    <integer>{{interval}}</integer>
    <key>StandardOutPath</key>
    <string>{{stdout}}</string>
    <key>StandardErrorPath</key>
    <string>{{stderr}}</string>
    <key>WorkingDirectory</key>
    <string>{{working_dir}}</string>
</dict>
</plist>";

fn main() {
    let args = Cli::from_args();

    if let Err(err) = run(args) {
        println!("Error: {}", err);
        std::process::exit(1);
    }
}

fn run(args: Cli) -> io::Result<()> {
    let cfg = match LaunchConfig::from_cli(&args) {
        Some(c) => c,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not generate launch config",
            ))
        }
    };

    let plist_file = match PlistFile::from(&cfg) {
        Some(plist) => plist,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "could not generate plist file",
            ))
        }
    };

    if args.dry_run {
        println!("Dry run: would write {}", plist_file);
        return Ok(());
    }

    cfg.dirs.ensure()?;
    plist_file.write()?;
    plist_file.load()?;
    println!("successfuly scheduled {}", cfg.name);

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
    period: Period,
    program: String,

    #[structopt(long)]
    dry_run: bool,

    #[structopt(long)]
    name: Option<String>,

    #[structopt(long)]
    args: Option<String>,

    #[structopt(long)]
    working_dir: Option<String>,
}

struct LaunchConfig {
    name: String,
    program_path: PathBuf,
    start_interval: u64,
    dirs: LaunchDirs,
    args: Vec<String>,
    working_dir: String,
}

impl LaunchConfig {
    fn from_cli(args: &Cli) -> Option<Self> {
        // First, try and treat the program as a filepath and see if we can
        // get the absolute path. Otherwise, we use the which crate to see
        // if the program matches an executable on the current PATH.
        let path = match fs::canonicalize(&args.program) {
            Ok(path) => path,
            Err(_) => match which(&args.program) {
                Ok(path) => path,
                Err(_) => return None,
            },
        };

        let name = match &args.name {
            Some(name) => name.to_owned(),
            None => path.file_stem()?.to_str()?.to_owned(),
        };

        let start_interval = args.period.to_seconds();
        let dirs = LaunchDirs::from(&name)?;

        let program_args = match &args.args {
            Some(a) => a.split_ascii_whitespace().map(|s| s.to_string()).collect(),
            None => Vec::new(),
        };

        let working_dir = match &args.working_dir {
            Some(dir) => dir.to_owned(),
            None => match env::current_dir() {
                Ok(dir) => dir.to_str()?.to_owned(),
                Err(_) => return None,
            },
        };

        Some(Self {
            name,
            program_path: path,
            start_interval,
            dirs,
            args: program_args,
            working_dir,
        })
    }

    fn plist_contents(&self) -> Option<String> {
        let program_path = self.program_path.to_str()?;
        let stdout_path = self.log_path("stdout")?;
        let stderr_path = self.log_path("stderr")?;

        let reg = Handlebars::new();
        match reg.render_template(
            PLIST_TEMPLATE,
            &json!(
                {
                    "name": self.name,
                    "program_path": program_path,
                    "args": self.args,
                    "interval": self.start_interval,
                    "stdout": stdout_path,
                    "stderr": stderr_path,
                    "working_dir": self.working_dir,
                }
            ),
        ) {
            Ok(contents) => Some(contents),
            Err(_) => None,
        }
    }

    fn log_path(&self, filename: &str) -> Option<String> {
        let mut path = self.dirs.log_dir.to_owned();
        path.push(filename);
        match path.to_str() {
            Some(p) => Some(p.to_owned()),
            None => None,
        }
    }

    fn plist_filepath(&self) -> Option<PathBuf> {
        let filename = format!("com.{}.plist", self.name);
        let mut filepath = dirs::home_dir()?;

        filepath.push("Library");
        filepath.push("LaunchAgents");
        filepath.push(filename);
        Some(filepath)
    }
}

struct LaunchDirs {
    log_dir: PathBuf,
    plist_dir: PathBuf,
}

impl LaunchDirs {
    fn from(name: &str) -> Option<Self> {
        let mut log_dir = dirs::home_dir()?;
        let mut plist_dir = log_dir.clone();

        log_dir.push("logs");
        log_dir.push(name);

        plist_dir.push("Library");
        plist_dir.push("LaunchAgents");

        Some(Self { log_dir, plist_dir })
    }

    fn ensure(&self) -> io::Result<()> {
        fs::create_dir_all(&self.log_dir)?;
        fs::create_dir_all(&self.plist_dir)?;
        Ok(())
    }
}

struct PlistFile {
    filepath: PathBuf,
    contents: String,
}

impl PlistFile {
    fn from(cfg: &LaunchConfig) -> Option<Self> {
        let filepath = cfg.plist_filepath()?;
        let contents = cfg.plist_contents()?;
        Some(Self { filepath, contents })
    }

    fn write(&self) -> io::Result<()> {
        fs::write(&self.filepath, &self.contents)
    }

    fn load(&self) -> io::Result<()> {
        let filepath = match self.filepath.to_str() {
            Some(path) => path,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "could not convert filepath to str",
                ))
            }
        };
        let status = Command::new("launchctl")
            .args(&["load", "-w", filepath])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to load plist file",
            ))
        }
    }
}

impl fmt::Display for PlistFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let filepath = match self.filepath.to_str() {
            Some(path) => path,
            None => return Err(fmt::Error),
        };

        write!(f, "{}:\n{}", filepath, self.contents)
    }
}
