use actix_web::{web, HttpResponse, Responder};
use std::io::{BufReader, Seek, SeekFrom, Read};
use std::fs::File;
use log::info;
use serde::Deserialize;
use serde_json::json;

use crate::service_info::ServiceInfo;

static CHUNK_SIZE: i64 = 1024;

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    lines: Option<usize>,
}

fn read_chunk(reader: &mut BufReader<File>, position: i64, size: i64) -> std::io::Result<Vec<u8>> {
    let mut chunk = vec![0; size as usize];
    reader.seek(SeekFrom::Start(position as u64))?;
    Read::read_exact(reader, &mut chunk)?;
    Ok(chunk)
}

fn read_last_lines(file_path: &str, num_lines: usize) -> std::io::Result<String> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let file_size = reader.seek(SeekFrom::End(0))?;
    let mut position = file_size as i64;
    // Create a string to hold a result lines as a string
    let mut lines = String::new();
    let mut found_lines = 0;

    while found_lines < num_lines {
        if position <= 0 { break; }
        position -= CHUNK_SIZE;

        // Read the chunk
        let chunk = match position < 0 {
            true => {
                read_chunk(&mut reader, 0, -position)?
            },
            false => {
                read_chunk(&mut reader, position, CHUNK_SIZE)?
            }
        };

        // Count the number of lines in the chunk by counting the number of newline characters
        let num_lines_in_chunk = chunk.iter().filter(|&&c| c == b'\n').count();
        found_lines += num_lines_in_chunk;

        if found_lines > num_lines {
            let over_lines = found_lines - num_lines;
            info!("Found {} lines in the chunk, over by {}", num_lines_in_chunk, over_lines);
            // Skip the first over_lines lines and join the rest
            let l = String::from_utf8(chunk).unwrap().lines().skip(over_lines).collect::<Vec<&str>>().join("\n");

            lines.insert_str(0, &l);
            return Ok(lines);
        } else {
            // Prepend the chunk to the lines
            lines.insert_str(0, &String::from_utf8(chunk).unwrap());
        }
    }

    Ok(lines)
}

pub async fn service_logs(path: web::Path<String>, query: web::Query<LogQuery>) -> impl Responder {
    let service_name = path.into_inner();
    let service_info = ServiceInfo::get_status(&service_name).unwrap();

    if service_info.log.is_none() {
        return HttpResponse::NotFound().body(format!("Can't find logs for service {}. Probably log service is not running", service_name));
    }

    let log_path = service_info.log.unwrap().log_directory().unwrap();
    let current_log_path = format!("{}/current", log_path);

    let lines = query.lines.unwrap_or(50);

    info!("Preparing for reading last {} lines from log file {}", lines, current_log_path);
    match read_last_lines(&current_log_path, lines) {
        Ok(logs) => HttpResponse::Ok().json(json!({ "logs": logs })),
        Err(_) => HttpResponse::NotFound().json(json!({ "error": format!("Could not read logs for service {}", service_name) })),
    }
}