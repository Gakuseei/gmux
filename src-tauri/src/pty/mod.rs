use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
pub struct PtyInstance {
    pub id: String,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    master: Arc<Mutex<Box<dyn portable_pty::MasterPty + Send>>>,
    child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    pub working_dir: String,
    pub command: String,
}

#[allow(dead_code)]
impl PtyInstance {
    pub fn write(&self, data: &[u8]) -> anyhow::Result<()> {
        self.writer
            .lock()
            .map_err(|e| anyhow::anyhow!("{e}"))?
            .write_all(data)?;
        Ok(())
    }

    pub fn resize(&self, rows: u16, cols: u16) -> anyhow::Result<()> {
        self.master
            .lock()
            .map_err(|e| anyhow::anyhow!("{e}"))?
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;
        Ok(())
    }

    pub fn kill(&self) -> anyhow::Result<()> {
        self.child
            .lock()
            .map_err(|e| anyhow::anyhow!("{e}"))?
            .kill()?;
        Ok(())
    }

    pub fn is_alive(&self) -> bool {
        self.child
            .lock()
            .ok()
            .and_then(|mut c| c.try_wait().ok())
            .map(|status| status.is_none())
            .unwrap_or(false)
    }
}

pub struct PtyManager {
    instances: HashMap<String, PtyInstance>,
}

#[allow(dead_code)]
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
        cmd.cwd(cwd);
        cmd.env("TERM", "xterm-256color");
        for (key, value) in &env_vars {
            cmd.env(key, value);
        }

        let child = pair.slave.spawn_command(cmd)?;
        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let id = uuid::Uuid::new_v4().to_string();

        let instance = PtyInstance {
            id: id.clone(),
            writer: Arc::new(Mutex::new(writer)),
            master: Arc::new(Mutex::new(pair.master)),
            child: Arc::new(Mutex::new(child)),
            working_dir: cwd.to_string(),
            command: shell.to_string(),
        };

        self.instances.insert(id.clone(), instance);

        Ok((id, reader))
    }

    pub fn write(&self, id: &str, data: &[u8]) -> anyhow::Result<()> {
        self.instances
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("pty {id} not found"))?
            .write(data)
    }

    pub fn resize(&self, id: &str, rows: u16, cols: u16) -> anyhow::Result<()> {
        self.instances
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("pty {id} not found"))?
            .resize(rows, cols)
    }

    pub fn kill(&mut self, id: &str) -> anyhow::Result<()> {
        self.instances
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("pty {id} not found"))?
            .kill()
    }

    pub fn remove(&mut self, id: &str) {
        self.instances.remove(id);
    }

    pub fn list(&self) -> Vec<String> {
        self.instances.keys().cloned().collect()
    }
}
