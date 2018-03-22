use aarch64;
use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};
use std::io::Write;
use std::str;
use std::path::{Path, PathBuf};
use fat32::vfat::*;
use fat32::traits::{FileSystem, Entry, Dir, Metadata, Timestamp};
use super::FILE_SYSTEM;

const SHELL_WELCOME: &'static str = r#"
  _____     _     _                ____   _____ 
 |_   _|   | |   (_)              / __ \ / ____|
   | |  ___| |__  _  __ _  ___   | |  | | (___  
   | | / __| '_ \| |/ _` |/ _ \  | |  | |\___ \ 
  _| || (__| | | | | (_| | (_) | | |__| |____) |
 |_____\___|_| |_|_|\__, |\___/   \____/|_____/ 
                     __/ |                      
                    |___/                       
"#;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }

    // Returns this command's arguments
    fn arguments(&self) -> &[&str] {
        &self.args[1..]
    }
}

// Registered command list
static SHELL_CMDS: &'static [&'static ShellCmd] = &[
    &EchoCmd,
    &PanicCmd,
    &AtagsCmd,
    &HeapTestCmd,
    &LsCmd,
    &CdCmd,
    &PwdCmd,
    &CatCmd,
    &CurrentELCmd
];

// Process a command received from shell
fn process_command(pwd: &mut PathBuf, cmd: Command) {
    let cmd_name = cmd.path();

    // Find the corresponding command
    // from the registered command list
    for shell_cmd in SHELL_CMDS {
        if shell_cmd.name() == cmd.path() {
            shell_cmd.exec(pwd, &cmd);
            return;
        }
    }

    kprintln!("error: unknown command: {}", cmd_name);
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// returns when exit is called.
/// TODO: implement `exit`
pub fn shell(prefix: &str) {
    // Print our awesome welcome message
    kprintln!("{}", SHELL_WELCOME);
    kprintln!("{}", "Welcome to Ichigo OS! 僕のダーリング。");
    kprintln!("");
    kprint!("{}", prefix);

    // Current working directory
    let mut pwd = PathBuf::new();
    pwd.push("/");

    // Use a StackVec for storage of command lines
    let mut line_buf = [0u8; 512];
    let mut line = StackVec::new(&mut line_buf[..]);
    loop {
        // Wait for the next byte to come in
        let byte = CONSOLE.lock().read_byte();

        if byte == b'\n' || byte == b'\r' {
            // Line break! We hopefully got a command!
            kprintln!("");
            {
                // Parse the string as UTF-8
                let line_str = str::from_utf8(&line);

                if let Ok(line_str) = line_str {
                    let mut cmd_buf = [""; 64];
                    let cmd = Command::parse(line_str, &mut cmd_buf[..]);
                    match cmd {
                        Ok(cmd) => process_command(&mut pwd, cmd),
                        Err(Error::TooManyArgs) => kprintln!("error: too many arguments"),
                        Err(Error::Empty) => ()
                    }
                    kprint!("{}", prefix);
                } else {
                    kprintln!("{}", "Illegal character detected.");
                }
            }

            // Clear buffer
            line.truncate(0);
        } else if byte == 8 || byte == 127 {
            // Backspace / delete character
            // Treat them all as backspace
            if !line.is_empty() {
                // Disallow backspacing through the prefix
                line.pop();

                // Back -> space -> back
                kprint!("\u{8} \u{8}");
            }
        } else if byte < 32 || byte > 127 {
            // Invisible characters in the ASCII range
            // or characters out of ASCII range
            // Yes, we can print UTF-8 characters but we cannot accept'em XD
            kprint!("\u{7}");
        } else if !line.is_full() {
            // Command not finished and we can still take more characters
            line.push(byte).unwrap();
            CONSOLE.lock().write(&[byte]).unwrap();
        }
    }
}

// Trait for a Shell command
trait ShellCmd: Sync + Send {
    // Returns the name (static) of the command
    fn name(&self) -> &'static str;

    // Called when the command is invoked via shell
    fn exec(&self, pwd: &mut PathBuf, args: &Command);
}

// $ echo a b c
// > a b c
struct EchoCmd;
impl ShellCmd for EchoCmd {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn exec(&self, pwd: &mut PathBuf, args: &Command) {
        for a in args.arguments() {
            kprint!("{} ", a);
        }
        kprintln!("");
    }
}

// Trigger kernel panic
struct PanicCmd;
impl ShellCmd for PanicCmd {
    fn name(&self) -> &'static str {
        "panic"
    }

    fn exec(&self, pwd: &mut PathBuf, args: &Command) {
        panic!("Requested panic")
    }
}

// Read Atags information from the memory
struct AtagsCmd;
impl ShellCmd for AtagsCmd {
    fn name(&self) -> &'static str {
        "atags"
    }

    fn exec(&self, pwd: &mut PathBuf, args: &Command) {
        use pi::atags;
    
        for tag in atags::Atags::get() {
            kprintln!("{:#?}", tag);
        }
    }
}

// Run a test against the heap
// $ heap_text x
// pushes x element into a vector
// and pop them into another
// then print the heap allocator status
struct HeapTestCmd;
impl ShellCmd for HeapTestCmd {
    fn name(&self) -> &'static str {
        "heap_test"
    }

    fn exec(&self, pwd: &mut PathBuf, args: &Command) {
        let args = args.arguments();
        if args.len() != 1 {
            kprintln!("error: incorrect number of arguments for `heap_test`");
            return;
        }
        let num = args[0].parse::<u32>();
        if let Ok(num) = num {
            kprintln!("> testing heap allocation...");
            let mut v = vec![];
            for i in 0..num {
                v.push(i);
            }
            kprintln!("> {:?}", v);
            let mut v2 = vec![];
            for i in 0..num {
                v2.push(v.pop());
            }
            kprintln!("> {:?}", v2);
            #[cfg(not(test))]
            kprintln!("{:#?}", super::ALLOCATOR);
        } else {
            kprintln!("error: cannot parse {} as number", args[0]);
        }
    }
}

// $ pwd
// print working directory
struct PwdCmd;
impl ShellCmd for PwdCmd {
    fn name(&self) -> &'static str {
        "pwd"
    }

    fn exec(&self, pwd: &mut PathBuf, args: &Command) {
        if args.arguments().len() > 0 {
            kprintln!("error: too many arguments");
            return;
        }

        kprintln!("{}", pwd.to_string_lossy());
    }
}

// $ cd dir
// change working directory into `dir`
struct CdCmd;
impl ShellCmd for CdCmd {
    fn name(&self) -> &'static str {
        "cd"
    }

    fn exec(&self, pwd: &mut PathBuf, args: &Command) {
        if args.arguments().len() != 1 {
            kprintln!("error: `cd` requires exactly 1 argument");
            return;
        }

        let dir = args.arguments()[0];
        match dir {
            ".." => {
                pwd.pop();
            },
            "." => {
                return;
            },
            dir => {
                pwd.push(dir);
                if FILE_SYSTEM.open(pwd.as_path()).is_err() {
                    pwd.pop();
                    kprintln!("error: {} not found", dir);
                }
            }
        }
    }
}

// List the entries in current directory
// $ ls [-a] [dir]
// -a: list hidden files
// dir: list files in dir instead of pwd
struct LsCmd;
impl ShellCmd for LsCmd {
    fn name(&self) -> &'static str {
        "ls"
    }

    fn exec(&self, pwd: &mut PathBuf, args: &Command) {
        let mut show_hidden = false;
        let mut custom_dir = false;
        let mut path = pwd.clone();
        for arg in args.arguments() {
            if arg == &"-a" {
                if !show_hidden {
                    show_hidden = true;
                } else {
                    kprintln!("error: multiple `-a` provided");
                    return;
                }
            } else {
                if !custom_dir {
                    custom_dir = true;
                    path.push(arg);
                } else {
                    kprintln!("error: too many arguments");
                    return;
                }
            }
        }
        let entry = FILE_SYSTEM.open(path.as_path());
        if entry.is_err() {
            kprintln!("error: cannot open {:?}", path);
            return;
        }

        let dir = entry.unwrap().into_dir();
        if dir.is_none() {
            kprintln!("error: {:?} is not a directory", path);
            return;
        }

        let entries = dir.unwrap().entries();
        if entries.is_err() {
            kprintln!("error: cannot list {:?}", path);
            return;
        }

        for item in entries.unwrap() {
            let mut name = item.name().to_string();

            if item.metadata().hidden() {
                if !show_hidden {
                    continue;
                } else {
                    name = format!("*{}", name);
                }
            }

            if item.as_dir().is_some() && name != "." && name != ".." {
                name = format!("{}/", name);
            }

            let timestamp = item.metadata().created();
            let time_format = format!("{}-{:02}-{:02} {:02}:{:02}:{:02}",
                timestamp.year(), timestamp.month(), timestamp.day(),
                timestamp.hour(), timestamp.minute(), timestamp.second());
            kprintln!("{}\t{}", time_format, name);
        }
    }
}

// $ cat file1 file2....
// print the content of file1, file2...
struct CatCmd;
impl CatCmd {
    fn print_file(pwd: &mut PathBuf, file: &str) {
        use std::io::Read;
        let mut path = pwd.clone();
        path.push(file);
        let entry = FILE_SYSTEM.open(path.as_path());
        if entry.is_err() {
            kprintln!("error: failed to open {:?}", path);
            return;
        }

        let entry = entry.unwrap();
        if let Some(mut f) = entry.into_file() {
            let mut buf = String::new();
            if f.read_to_string(&mut buf).is_ok() {
                kprint!("{}", buf);
            } else {
                kprintln!("error: unable to read from {:?}", path);
            }
        } else {
            kprintln!("error: {:?} is a directory", path);
        }
    }
}
impl ShellCmd for CatCmd {
    fn name(&self) -> &'static str {
        "cat"
    }

    fn exec(&self, pwd: &mut PathBuf, args: &Command) {
        if args.arguments().len() == 0 {
            kprintln!("error: `cat` requires at least one argument");
            return;
        }

        for file in args.arguments() {
            CatCmd::print_file(pwd, file);
        }
    }
}

// $ current_el
// print the current exception level
struct CurrentELCmd;
impl ShellCmd for CurrentELCmd {
    fn name(&self) -> &'static str {
        "current_el"
    }

    fn exec(&self, pwd: &mut PathBuf, args: &Command) {
        let el = unsafe {
            aarch64::current_el()
        };
        kprintln!("current excepton level: {}", el);
    }
}