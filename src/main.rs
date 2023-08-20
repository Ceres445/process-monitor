use std::{process::Command, time::SystemTime};

use clap::Parser;
use csv::Writer;
use sysinfo::{Pid, PidExt, ProcessExt, SystemExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    pid_or_command: String,
    logfile: String,
    #[arg(short, long)]
    interval: Option<u32>,
    #[arg(short, long)]
    duration: Option<u32>,
    #[arg(short, long)]
    network: bool,
}

fn main() {
    let args = Cli::parse();
    // println!("{0}", args.pattern);
    // println!("{0}", args.path.display());

    let pid = match args.pid_or_command.parse::<u32>() {
        Ok(pid) => pid,
        Err(_) => {
            let child = Command::new(args.pid_or_command.clone())
                .spawn()
                .expect("failed to execute command");
            child.id()
        }
    };

    monitor(
        Pid::from_u32(pid),
        args.logfile,
        args.interval,
        args.duration,
        args.network,
    );
}

fn monitor(
    pid: Pid,
    filename: String,
    interval: Option<u32>,
    duration: Option<u32>,
    include_network: bool,
) {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    let start_time = SystemTime::now();
    let mut csv_writer = Writer::from_path(filename).expect("Unable to create csv file");
    if include_network {
        csv_writer
            .write_record(&[
                "Elapsed time",
                "CPU (%)",
                "Real memory (MB)",
                "Virtual memory (MB)",
                "IO Write (MB)",
                "IO Read (MB)",
                "Network Upload (MB)",
                "Network Download (MB)",
            ])
            .expect("Unable to write to csv file");
    } else {
        csv_writer
            .write_record(&[
                "Elapsed time",
                "CPU (%)",
                "Real memory (MB)",
                "Virtual memory (MB)",
                "IO Write (MB)",
                "IO Read (MB)",
            ])
            .expect("Unable to write to csv file");
    }
    loop {
        let process = system
            .process(pid)
            .expect("Unable to find process with pid");
        let io = process.disk_usage();
        match duration {
            Some(duration) => {
                if SystemTime::now()
                    .duration_since(start_time)
                    .expect("Time went backwards")
                    .as_secs()
                    > duration as u64
                {
                    break;
                }
            }
            None => {}
        }
        match process.status() {
            sysinfo::ProcessStatus::Zombie => break,
            sysinfo::ProcessStatus::Dead => break,
            _ => {}
        }
        csv_writer
            .write_record(&[
                SystemTime::now()
                    .duration_since(start_time)
                    .expect("Time went backwards")
                    .as_secs()
                    .to_string(),
                format!("{:.2}", process.cpu_usage()),
                (process.memory() / 1024 / 1024).to_string(),
                (process.virtual_memory() / 1024 / 1024).to_string(),
                (io.total_written_bytes / 1024 / 1024).to_string(),
                (io.total_read_bytes / 1024 / 1024).to_string(),
            ])
            .expect("Unable to write to csv file");
        println!(
            "{} {} {} {} {} {}",
            SystemTime::now()
                .duration_since(start_time)
                .expect("Time went backwards")
                .as_secs(),
            format!("{:.2}", process.cpu_usage()),
            process.memory() / 1024 / 1024,
            process.virtual_memory() / 1024 / 1024,
            io.total_written_bytes.to_string(),
            io.total_read_bytes.to_string(),
        );
        // Writes to file in case program shuts down before expected
        csv_writer.flush().expect("Unable to flush csv file");
        // Sleeps for interval seconds
        if let Some(interval) = interval {
            std::thread::sleep(std::time::Duration::from_secs(interval as u64));
        }
        // Refreshes system so that stats are updated
        system.refresh_process(pid);
    }
}
