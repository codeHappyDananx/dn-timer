#![allow(dead_code)]

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct PythonBridge {
    #[allow(dead_code)]
    child: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<std::process::ChildStdin>>,
    stdout_reader: Arc<Mutex<BufReader<std::process::ChildStdout>>>,
}

/// Sidecar 查找结果：可执行文件路径 + 启动参数
struct SidecarCommand {
    program: std::path::PathBuf,
    args: Vec<String>,
}

impl PythonBridge {
    pub fn new() -> anyhow::Result<Self> {
        let sidecar = Self::find_sidecar()?;

        let mut cmd = Command::new(&sidecar.program);
        for arg in &sidecar.args {
            cmd.arg(arg);
        }

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to start sidecar at {:?}: {}. Please build the sidecar first.",
                    sidecar.program,
                    e
                )
            })?;

        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = child.stdout.take().expect("piped stdout");
        let stdout_reader = BufReader::new(stdout);

        // 在后台线程中读取 stderr 并打印
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    eprintln!("[sidecar stderr] {}", line);
                }
            });
        }

        Ok(Self {
            child: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            stdout_reader: Arc::new(Mutex::new(stdout_reader)),
        })
    }

    fn find_sidecar() -> anyhow::Result<SidecarCommand> {
        // 1. 先检查 exe 同级目录（生产环境，Tauri externalBin 会放在这里）
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let p = dir.join("python-ocr-sidecar.exe");
                if p.exists() {
                    return Ok(SidecarCommand {
                        program: p,
                        args: vec![],
                    });
                }
            }
        }

        // 2. 检查项目根目录的 sidecar/main.py（开发环境）
        let dev_py = std::path::PathBuf::from("sidecar/main.py");
        if dev_py.exists() {
            return Ok(SidecarCommand {
                program: std::path::PathBuf::from("python"),
                args: vec!["sidecar/main.py".to_string()],
            });
        }

        // 3. 尝试用系统 python 运行
        if let Ok(output) = std::process::Command::new("python")
            .arg("--version")
            .output()
        {
            if output.status.success() {
                let py_path = std::path::PathBuf::from("sidecar/main.py");
                if py_path.exists() {
                    return Ok(SidecarCommand {
                        program: std::path::PathBuf::from("python"),
                        args: vec!["sidecar/main.py".to_string()],
                    });
                }
            }
        }

        Err(anyhow::anyhow!(
            "Sidecar not found. Please build or install python sidecar."
        ))
    }

    pub fn request(&self, req: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let req_line = serde_json::to_string(&req)?;

        {
            let mut stdin = self.stdin.lock().unwrap();
            writeln!(stdin, "{}", req_line)?;
            stdin.flush()?;
        }

        let mut resp_line = String::new();
        {
            let mut reader = self.stdout_reader.lock().unwrap();
            reader.read_line(&mut resp_line)?;
        }

        let resp_line = resp_line.trim();
        if resp_line.is_empty() {
            anyhow::bail!("Empty response from sidecar");
        }

        let resp: serde_json::Value = serde_json::from_str(resp_line).map_err(|e| {
            anyhow::anyhow!("Invalid JSON from sidecar: {} | raw: {}", e, resp_line)
        })?;
        Ok(resp)
    }

    pub fn recognize(
        &self,
        image_path: Option<&str>,
        character_name: Option<&str>,
    ) -> anyhow::Result<serde_json::Value> {
        let req = serde_json::json!({
            "method": "recognize",
            "params": {
                "image_path": image_path,
                "character_name": character_name.unwrap_or("")
            }
        });
        self.request(req)
    }
}

impl Drop for PythonBridge {
    fn drop(&mut self) {
        // 优雅终止 sidecar 进程：关闭 stdin 后尝试 kill
        let _ = self.stdin.lock().unwrap().flush();
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
        }
    }
}
