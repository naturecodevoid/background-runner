use std::{
    env::args,
    io::{stdin, stdout, Write},
    process::Command,
    thread,
    time::Duration,
};

use yansi::Paint;

fn main() {
    // yansi::whenever(yansi::Condition::TTY_AND_COLOR);

    let mut args = args();
    args.next();
    let Some(command) = args.next() else {
        println!("{}: an extremely simple Rust program that runs a command in the background, while allowing the user to restart it at any time.
Homepage: <{}>

Example: `background-runner sleep 10`", "background-runner".bold().green(), "https://github.com/naturecodevoid/background-runner".blue());
        println!();
        println!("{}: Expected at least one argument (the command to run)", "error".red());
        return;
    };

    let args = {
        let args: Vec<_> = args.collect();
        if args.is_empty() {
            None
        } else {
            Some(args)
        }
    };

    println!("{}: {command}", "command".blue());
    println!(
        "{}: {:?}",
        "args".blue(),
        args.as_ref().map(|a| a.as_slice()).unwrap_or(&[])
    );
    println!();
    println!(
        "{}",
        "press Enter to restart the process (including while it is running)".blue()
    );
    println!("{}", "press Ctrl+C to exit background-runner".blue());
    println!();

    let (stdin_sender, stdin_receiver) = crossbeam_channel::bounded(0);

    thread::spawn(move || {
        let mut lines = stdin().lines();
        loop {
            // .next() is blocking, no need to thread::sleep()
            if lines.next().is_some() {
                stdin_sender.send(()).unwrap();
            }
        }
    });

    loop {
        let mut command = Command::new(&command);
        if let Some(args) = &args {
            command.args(args);
        }
        println!("{}", "Starting process".green());
        let mut child = match command.spawn() {
            Ok(c) => c,
            Err(e) => {
                println!("\n{}: Failed to start process: {e}", "error".red());
                break;
            }
        };

        let mut completed = false;
        // Wait for enter press
        while stdin_receiver.try_recv().is_err() {
            if !completed && child.try_wait().ok().and_then(|s| s).is_some() {
                completed = true;
                print!("{}", "Process has completed, press Enter to run it again or press Ctrl+C to exit background-runner".yellow());
                stdout().flush().unwrap();
            }
            thread::sleep(Duration::from_millis(200)); // Help with CPU usage; 100 milliseconds is low enough for <0.0% CPU usage but 200 feels exactly as responsive
        }
        if !completed {
            println!("{}", "Killing process".red());
            child.kill().unwrap();
        }
    }
}
