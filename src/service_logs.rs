use actix_web::{web, HttpResponse, Responder};
use std::fs::File;
use std::io::{self, BufRead, Seek, SeekFrom};

fn read_last_lines(file_path: &str, num_lines: usize) -> io::Result<String> {
    let file = File::open(file_path)?;
    let mut reader = io::BufReader::new(file);

    let mut lines = Vec::new();
    let mut position = reader.seek(SeekFrom::End(0))?;

    while lines.len() < num_lines && position > 0 {
        position -= 1;
        reader.seek(SeekFrom::Start(position))?;
        if reader.fill_buf()?.starts_with(b"\n") {
            let mut line = String::new();
            reader.read_line(&mut line)?;
            lines.push(line);
        }
    }

    Ok(lines.into_iter().rev().collect::<String>())
}

pub async fn service_logs(path: web::Path<String>) -> impl Responder {
    let service_name = path.into_inner();
    let log_path = format!("/etc/service/{}/log/current", service_name);

    match read_last_lines(&log_path, 50) {
        Ok(logs) => HttpResponse::Ok().body(logs),
        Err(_) => HttpResponse::NotFound().body(format!("Could not read logs for service {}", service_name)),
    }
}
