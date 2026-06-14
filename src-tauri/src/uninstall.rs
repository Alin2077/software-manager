use std::process::Command;

/// 运行卸载命令
pub fn run_uninstall_string(command: &str) {
    // 卸载命令可能是 exe 路径或带参数的命令
    // 尝试用 cmd /c 来执行以处理各种格式
    let mut cmd = Command::new("cmd");
    cmd.arg("/c").arg(command);
    // 不等待卸载完成（卸载可能要很久，且会等待用户交互）
    cmd.spawn().ok();
}
