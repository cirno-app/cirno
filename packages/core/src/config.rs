use std::path::PathBuf;

pub struct EnvironmentState {
    exe_dir: PathBuf,
}

pub fn load_config(exe_dir: PathBuf) -> EnvironmentState {
    EnvironmentState { exe_dir }
}
