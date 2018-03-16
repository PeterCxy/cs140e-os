use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};
use std::io::Write;
use std::str;

const SHELL_WELCOME: &'static str = "
  _____     _     _                ____   _____ \r
 |_   _|   | |   (_)              / __ \\ / ____|\r
   | |  ___| |__  _  __ _  ___   | |  | | (___  \r
   | | / __| '_ \\| |/ _` |/ _ \\  | |  | |\\___ \\ \r
  _| || (__| | | | | (_| | (_) | | |__| |____) |\r
 |_____\\___|_| |_|_|\\__, |\\___/   \\____/|_____/ \r
                     __/ |                      \r
                    |___/                       \r
";

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

// > $ echo $a $b $c
// > $a $b $c
// TODO: better implementation of command registry
fn cmd_echo(cmd: &Command) {
    for a in cmd.arguments() {
        kprint!("{} ", a);
    }
    kprintln!("");
}

// Process a command received from shell
// TODO: Better implementation
fn process_command(cmd: Command) {
    match cmd.path() {
        "echo" => cmd_echo(&cmd),
        p => kprintln!("unknown command: {}", p)
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) -> ! {
    // Print our awesome welcome message
    kprintln!("{}", SHELL_WELCOME);
    kprintln!("{}", "Welcome to Ichigo OS! 僕のダーリング。");
    kprintln!("");
    kprint!("{}", prefix);

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
                        Ok(cmd) => process_command(cmd),
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