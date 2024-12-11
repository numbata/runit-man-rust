use std::io::BufReader;
use std::fs::File;
use log::info;
use rev_lines::RevLines;

use crate::application::service_info::ServiceInfo;

fn read_last_lines(file_path: &str, num_lines: usize) -> std::io::Result<String> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let rev_lines = RevLines::new(reader);

    // rev_lines yields lines from the end. Take the desired number of lines.
    let mut collected: Vec<String> = rev_lines.take(num_lines)
        .filter_map(Result::ok)
        .collect();
    collected.reverse();

    Ok(collected.join("\n"))
}

pub fn service_logs(service_info: ServiceInfo, lines: usize) -> std::io::Result<String> {
    let log_path = service_info.log.unwrap().log_directory().unwrap();
    let current_log_path = format!("{}/current", log_path);

    info!("Preparing for reading last {} lines from log file {}", lines, current_log_path);
    read_last_lines(&current_log_path, lines)
}