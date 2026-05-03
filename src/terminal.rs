use std::{
    env,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::mpsc::Sender,
    thread,
};

use anyhow::{Context, Result};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

use crate::session::SessionTab;

pub(crate) struct TerminalTab {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) cwd: PathBuf,
    pub(crate) shell: String,
    parser: vt100::Parser,
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
}

impl TerminalTab {
    pub(crate) fn spawn(
        id: u64,
        name: String,
        cwd: PathBuf,
        shell: String,
        cols: u16,
        rows: u16,
        output_tx: Sender<(u64, Vec<u8>)>,
    ) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("failed to open pty")?;

        let mut command = CommandBuilder::new(&shell);
        command.cwd(&cwd);

        let child = pair
            .slave
            .spawn_command(command)
            .with_context(|| format!("failed to spawn shell {shell}"))?;
        let mut reader = pair
            .master
            .try_clone_reader()
            .context("failed to clone pty reader")?;
        let writer = pair
            .master
            .take_writer()
            .context("failed to open pty writer")?;

        thread::spawn(move || {
            let mut buffer = [0_u8; 8192];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        if output_tx.send((id, buffer[..n].to_vec())).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            id,
            name,
            cwd,
            shell,
            parser: vt100::Parser::new(rows, cols, 10_000),
            writer,
            master: pair.master,
            child,
        })
    }

    pub(crate) fn from_session(
        id: u64,
        tab: SessionTab,
        cols: u16,
        rows: u16,
        output_tx: Sender<(u64, Vec<u8>)>,
    ) -> Result<Self> {
        Self::spawn(id, tab.name, tab.cwd, tab.shell, cols, rows, output_tx)
    }

    pub(crate) fn new_default(
        id: u64,
        cwd: PathBuf,
        name: String,
        cols: u16,
        rows: u16,
        output_tx: Sender<(u64, Vec<u8>)>,
    ) -> Result<Self> {
        Self::spawn(id, name, cwd, default_shell(), cols, rows, output_tx)
    }

    pub(crate) fn write_input(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer
            .write_all(bytes)
            .context("failed to write to pty")?;
        self.writer.flush().context("failed to flush pty input")
    }

    pub(crate) fn process_output(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub(crate) fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("failed to resize pty")?;
        self.parser.set_size(rows, cols);
        Ok(())
    }

    pub(crate) fn screen(&self) -> &vt100::Screen {
        self.parser.screen()
    }

    pub(crate) fn session_tab(&self) -> SessionTab {
        SessionTab {
            name: self.name.clone(),
            cwd: self.cwd.clone(),
            shell: self.shell.clone(),
        }
    }
}

impl Drop for TerminalTab {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub(crate) fn default_shell() -> String {
    env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
}

pub(crate) fn default_cwd() -> Result<PathBuf> {
    env::current_dir().context("failed to resolve current directory")
}

pub(crate) fn normalize_cwd(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
