use std::process::Command;

fn run_shell(shell_name: &str, command: &str) -> Option<String> {
    let args = match shell_name {
        "powershell" => ["-Command", command],
        "cmd" => ["/C", command],
        _ => ["-c", command],
    };
    let output = Command::new(shell_name)
        .args(args)
        .output()
        .expect("Failed to execute command");
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).expect("Not UTF-8");
        let stdout = stdout.trim().to_string();
        return Some(stdout);
    }
    return None;
}

pub fn run_powershell(command: &str) -> Option<String> {
    run_shell("powershell", command)
}

pub fn run_cmd(command: &str) -> Option<String> {
    run_shell("cmd", command)
}