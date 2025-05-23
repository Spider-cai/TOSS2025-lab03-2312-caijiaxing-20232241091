use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::path::Path;
use std::env;

fn main() {
    loop {
        // 打印提示符
        print_prompt();
        
        // 读取输入
        let input = match read_input() {
            Ok(input) => input,
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        };
        
        // 解析并执行命令
        if let Err(e) = parse_and_execute(&input) {
            eprintln!("Error: {}", e);
        }
    }
}

fn print_prompt() {
    // 获取当前工作目录
    let current_dir = env::current_dir().unwrap_or_default();
    let current_dir = current_dir.to_string_lossy();
    
    // 获取用户名和主机名
    let username = whoami::username();
    //let hostname = whoami::hostname();
    let hostname = whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string());
    print!("{}@{}:{} $ ", username, hostname, current_dir);
    io::stdout().flush().unwrap();
}

fn read_input() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn parse_and_execute(input: &str) -> io::Result<()> {
    if input.is_empty() {
        return Ok(());
    }
    
    // 分割管道命令
    let commands: Vec<&str> = input.split('|').map(|s| s.trim()).collect();
    
    if commands.len() > 1 {
        execute_pipeline(&commands)?;
        return Ok(());
    }
    
    // 处理单个命令
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }
    
    match parts[0] {
        "cd" => {
            if parts.len() > 1 {
                let path = parts[1];
                change_directory(path)?;
            } else {
                change_directory("~")?;
            }
        }
        "exit" => std::process::exit(0),
        command => execute_command(command, &parts[1..])?,
    }
    
    Ok(())
}

fn change_directory(path: &str) -> io::Result<()> {
    let path = if path == "~" {
        dirs::home_dir().ok_or_else(|| 
            io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?
    } else {
        Path::new(path).to_path_buf()
    };
    
    env::set_current_dir(path)?;
    Ok(())
}

fn execute_command(command: &str, args: &[&str]) -> io::Result<()> {
    let mut cmd = Command::new(command);
    
    cmd.args(args)
       .stdin(Stdio::inherit())
       .stdout(Stdio::inherit())
       .stderr(Stdio::inherit());
    
    let status = cmd.status()?;
    
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Command failed with exit code: {:?}", status.code())
        ));
    }
    
    Ok(())
}

fn execute_pipeline(commands: &[&str]) -> io::Result<()> {
    let mut previous_output = None;
    
    for (i, cmd_str) in commands.iter().enumerate() {
        let parts: Vec<&str> = cmd_str.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        let mut cmd = Command::new(parts[0]);
        cmd.args(&parts[1..]);
        
        // 设置输入
        if let Some(prev_out) = previous_output {
            cmd.stdin(prev_out);
        }
        
        // 设置输出
        cmd.stdout(if i < commands.len() - 1 {
            Stdio::piped()
        } else {
            Stdio::inherit()
        });
        
        let child = cmd.spawn()?;
        
        // 更新 previous_output 而不产生借用冲突
        previous_output = child.stdout;
    }
    
    Ok(())
}

// fn execute_pipeline(commands: &[&str]) -> io::Result<()> {
//     let mut previous_output = None;
    
//     for (i, cmd_str) in commands.iter().enumerate() {
//         let parts: Vec<&str> = cmd_str.split_whitespace().collect();
//         if parts.is_empty() {
//             continue;
//         }
        
//         let mut cmd = Command::new(parts[0]);
//         cmd.args(&parts[1..]);
        
//         // 设置输入输出
//         if let Some(prev_out) = previous_output {
//             cmd.stdin(prev_out);
//         }
        
//         // 如果不是最后一个命令，准备管道输出
//         if i < commands.len() - 1 {
//             cmd.stdout(Stdio::piped());
//         } else {
//             cmd.stdout(Stdio::inherit());
//         }
        
//         let child = cmd.spawn()?;
        
//         // 关闭前一个命令的输出（当前命令已经接管）
//         if let Some(prev_out) = previous_output.take() {
//             drop(prev_out);
//         }
        
//         previous_output = child.stdout;
//     }
    
//     Ok(())
// }
// #[cfg(test)]
// mod tests {
//      //use super::*;
//     //use std::process::Output;
//     use std::process::Command as StdCommand;

//     fn run_shell_command(cmd: &str) -> String {
//         let output = StdCommand::new("sh")
//             .arg("-c")
//             .arg(cmd)
//             .output()
//             .expect("Failed to execute command");
//         String::from_utf8(output.stdout).unwrap()
//     }

//     #[test]
//     fn test_basic_command() {
//         let output = run_shell_command("echo hello");
//         assert_eq!(output.trim(), "hello");
//     }

//     #[test]
//     fn test_pipeline() {
//         let output = run_shell_command("ls | head -n 3 | wc -l");
//         let count = output.trim().parse::<i32>().unwrap();
//         assert!(count <= 3);
//     }
// }