use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};

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
        self.child.kill()?;
        Ok(())
    }
}

pub struct PtyManager {
    instances: HashMap<String, PtyInstance>,
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

        let instance = PtyInstance {
            writer,
            master: pair.master,
            child,
        };

        self.instances.insert(id.clone(), instance);

        Ok((id, reader))
    }

    pub fn write(&mut self, id: &str, data: &[u8]) -> anyhow::Result<()> {
        self.instances
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("pty {id} not found"))?
            .write(data)
    }

    pub fn resize(&mut self, id: &str, rows: u16, cols: u16) -> anyhow::Result<()> {
        self.instances
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("pty {id} not found"))?
            .resize(rows, cols)
    }

    pub fn kill(&mut self, id: &str) -> anyhow::Result<()> {
        self.instances
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("pty {id} not found"))?
            .kill()
    }

    pub fn remove(&mut self, id: &str) {
        self.instances.remove(id);
    }
}
