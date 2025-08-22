use anyhow::Result;
use std::{ffi::OsStr, path::Path, process::ExitStatus};
use tokio::process::{Child, Command};

use crate::config::EnvironmentState;

/// Cirno apps child process.
///
/// Due to its importance, we decided to name it `CirnoProc` instead of Proc.
pub struct CirnoProc {
    cmd: Command,
    child: Option<Child>,
}

impl CirnoProc {
    pub fn new<SP: AsRef<OsStr>, IA: IntoIterator<Item = SA>, SA: AsRef<OsStr>, P: AsRef<Path>>(
        program: SP,
        args: IA,
        cwd: P,
    ) -> CirnoProc {
        let mut proc = Command::new(program);
        proc.args(args);
        proc.current_dir(cwd);

        CirnoProc {
            cmd: proc,
            child: None,
        }
    }

    pub fn new_node<IA: IntoIterator<Item = SA>, SA: AsRef<OsStr>, P: AsRef<Path>>(
        env: &EnvironmentState,
        args: IA,
        cwd: P,
    ) -> CirnoProc {
        CirnoProc::new(env.bin_dir.join("node"), args, cwd)
    }

    pub fn new_yarn<P: AsRef<Path>>(
        env: &EnvironmentState,
        args: &Vec<&OsStr>,
        cwd: P,
    ) -> CirnoProc {
        let mut args = args.clone();
        let yarn_path = env.bin_dir.join("yarn.cjs");
        args.insert(0, yarn_path.as_os_str());

        CirnoProc::new_node(env, args, cwd)
    }

    pub async fn run(&mut self) -> Result<()> {
        self.child = Some(self.cmd.spawn()?);

        let child = self.child.as_mut().unwrap();

        let exit_status = child.wait().await?;

        self.child = None;

        exit_status.exit_ok()?;

        Ok(())
    }
}
