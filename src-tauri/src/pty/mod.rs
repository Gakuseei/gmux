use nix::sys::signal::{kill as nix_kill, Signal};
use nix::unistd::Pid;
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

pub struct PtyInstance {
    writer: Box<dyn Write + Send>,
    master: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl PtyInstance {
    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.writer.write_all(data)?;
        Ok(())
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> anyhow::Result<()> {
        self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        Ok(())
    }

    pub fn kill(&mut self) -> anyhow::Result<()> {
        if let Some(pid) = self.child.process_id() {
            let nix_pid = Pid::from_raw(pid as i32);
            if nix_kill(nix_pid, Signal::SIGTERM).is_ok() {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if self.child.try_wait().ok().flatten().is_none() {
                    self.child.kill()?;
                }
                return Ok(());
            }
        }
        self.child.kill()?;
        Ok(())
    }
}

pub struct PtyManager {
    instances: HashMap<String, Arc<Mutex<PtyInstance>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            instances: HashMap::new(),
        }
    }

    pub fn spawn(
        &mut self,
        shell: &str,
        cwd: &str,
        cols: u16,
        rows: u16,
        env_vars: Vec<(String, String)>,
    ) -> anyhow::Result<(String, Box<dyn Read + Send>)> {
        let pty_system = native_pty_system();

        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new(shell);
        let resolved_cwd = if cwd.is_empty() || cwd == "~" {
            dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("/"))
                .to_string_lossy()
                .to_string()
        } else {
            cwd.to_string()
        };
        cmd.cwd(&resolved_cwd);
        cmd.env("TERM", "xterm-256color");
        for (key, value) in &env_vars {
            cmd.env(key, value);
        }

        let child = pair.slave.spawn_command(cmd)?;
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let id = uuid::Uuid::new_v4().to_string();

        let instance = Arc::new(Mutex::new(PtyInstance {
            writer,
            master: pair.master,
            child,
        }));

        self.instances.insert(id.clone(), instance);

        Ok((id, reader))
    }

    pub fn get(&self, id: &str) -> anyhow::Result<Arc<Mutex<PtyInstance>>> {
        self.instances
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("pty {id} not found"))
    }

    pub fn remove(&mut self, id: &str) {
        self.instances.remove(id);
    }
}
