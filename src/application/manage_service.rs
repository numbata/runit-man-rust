use std::os::unix::fs::symlink;
use std::fs::remove_file;
use std::process::{Command, Output};
use std::io;


#[derive(Debug)]
enum ServiceAction {
    Start,
    Stop,
    Restart,
    Enable,
    Disable,
}

impl ServiceAction {
    fn from_str(action: &str) -> Option<Self> {
        match action {
            "start" => Some(ServiceAction::Start),
            "stop" => Some(ServiceAction::Stop),
            "restart" => Some(ServiceAction::Restart),
            "enable" => Some(ServiceAction::Enable),
            "disable" => Some(ServiceAction::Disable),
            _ => None,
        }
    }
}

fn execute_command(command: &mut Command) -> Result<Output, io::Error> {
    let output = command.output()?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Command execution failed: {:?} with status: {}",
                command,
                output.status
            ),
        ))
    }
}

pub fn perform_service_action(service_name: &str, action: &str) -> Result<String, Box<dyn std::error::Error>> {
    let action = ServiceAction::from_str(action);

    match action {
        Some(ServiceAction::Start) => {
            execute_command(Command::new("sv").arg("up").arg(service_name))?;
            Ok(format!("Service {} started.", service_name))
        },
        Some(ServiceAction::Stop) => {
            execute_command(Command::new("sv").arg("down").arg(service_name))?;
            Ok(format!("Service {} stopped.", service_name))
        },
        Some(ServiceAction::Restart) => {
            execute_command(Command::new("sv").arg("restart").arg(service_name))?;
            Ok(format!("Service {} restarted.", service_name))
        },
        Some(ServiceAction::Enable) => {
            let source = format!("/etc/sv/{}", service_name);
            let target = format!("/etc/service/{}", service_name);
            symlink(source, target)?;
            Ok(format!("Service {} enabled.", service_name))
        },
        Some(ServiceAction::Disable) => {
            let target = format!("/etc/service/{}", service_name);
            remove_file(target)?;
            Ok(format!("Service {} disabled.", service_name))
        },
        None => Err(Box::<dyn std::error::Error>::from("Invalid action")),
    }
}
