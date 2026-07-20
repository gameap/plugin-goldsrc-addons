//! Seam between handlers and the SDK host functions.
//!
//! `gameap_plugin_sdk::host` is compiled only for wasm32, so handlers depend
//! on this trait instead; native `cargo test` runs them against `MockHost`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostApiError {
    /// ABI/transport failure of the host call itself.
    Call(String),
    /// The daemon reported a failed operation (`success: false` / `error`).
    Op(String),
}

pub type HostResult<T> = Result<T, HostApiError>;

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub id: u64,
    pub game_code: String,
    pub node_id: u64,
    pub dir: String,
}

#[derive(Debug, Clone)]
pub struct GameInfo {
    pub code: String,
    pub name: String,
    pub engine: String,
}

#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: u64,
    pub os: String,
    pub work_path: String,
}

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct FileStat {
    pub is_dir: bool,
    pub size: u64,
    /// Unix permission bits (0 when the node does not report them).
    pub permissions: u32,
}

/// Result of a command executed on a node (nodecmd).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub output: String,
    pub exit_code: i32,
}

pub trait HostApi {
    fn get_server(&mut self, id: u64) -> HostResult<Option<ServerInfo>>;
    fn get_game(&mut self, code: &str) -> HostResult<Option<GameInfo>>;
    fn get_node(&mut self, id: u64) -> HostResult<Option<NodeInfo>>;
    /// `Ok(None)` — directory missing or unreadable.
    fn read_dir(&mut self, node_id: u64, path: &str) -> HostResult<Option<Vec<DirEntry>>>;
    /// Existence probe; `Ok(None)` — not found.
    fn stat(&mut self, node_id: u64, path: &str) -> HostResult<Option<FileStat>>;
    fn download(&mut self, node_id: u64, path: &str) -> HostResult<Vec<u8>>;
    fn upload(&mut self, node_id: u64, path: &str, content: &[u8], permissions: u32)
    -> HostResult<()>;
    fn remove(&mut self, node_id: u64, path: &str, recursive: bool) -> HostResult<()>;
    fn chmod(&mut self, node_id: u64, path: &str, permissions: u32) -> HostResult<()>;
    fn execute_command(
        &mut self,
        node_id: u64,
        command: &str,
        work_dir: Option<&str>,
    ) -> HostResult<CommandOutput>;
    fn log_info(&mut self, message: &str);
    fn log_error(&mut self, message: &str);
}

pub struct WasmHost;

/// HashMap-backed fake node used by native handler tests.
#[cfg(test)]
pub mod mock {
    use std::collections::{BTreeMap, BTreeSet, VecDeque};

    use super::*;

    #[derive(Default)]
    pub struct MockHost {
        pub servers: BTreeMap<u64, ServerInfo>,
        pub games: BTreeMap<String, GameInfo>,
        pub nodes: BTreeMap<u64, NodeInfo>,
        /// Absolute path → file content.
        pub files: BTreeMap<String, Vec<u8>>,
        /// Absolute path → unix permission bits (files default to 0o644).
        pub perms: BTreeMap<String, u32>,
        /// Absolute paths of directories.
        pub dirs: BTreeSet<String>,
        /// chmod calls, in call order.
        pub chmods: Vec<(String, u32)>,
        /// Commands passed to execute_command, in call order.
        pub commands: Vec<(String, Option<String>)>,
        /// Canned execute_command results, popped one per call.
        pub command_results: VecDeque<CommandOutput>,
        pub logs: Vec<String>,
    }

    impl MockHost {
        /// A CS 1.6 server (id 3) on node 1 rooted at /srv/gameap/servers/cs.
        pub fn goldsource() -> MockHost {
            let mut host = MockHost::default();
            host.servers.insert(
                3,
                ServerInfo {
                    id: 3,
                    game_code: "cstrike".into(),
                    node_id: 1,
                    dir: "servers/cs".into(),
                },
            );
            host.games.insert(
                "cstrike".into(),
                GameInfo {
                    code: "cstrike".into(),
                    name: "Counter-Strike 1.6".into(),
                    engine: "GoldSource".into(),
                },
            );
            host.nodes.insert(
                1,
                NodeInfo {
                    id: 1,
                    os: "linux".into(),
                    work_path: "/srv/gameap".into(),
                },
            );
            host.add_dir(MockHost::MOD_ABS);
            host
        }

        pub const MOD_ABS: &str = "/srv/gameap/servers/cs/cstrike";

        pub fn add_dir(&mut self, path: &str) {
            let mut current = String::new();
            for segment in path.trim_start_matches('/').split('/') {
                current.push('/');
                current.push_str(segment);
                self.dirs.insert(current.clone());
            }
        }

        pub fn add_file(&mut self, path: &str, content: &[u8]) {
            if let Some(idx) = path.rfind('/') {
                self.add_dir(&path[..idx]);
            }
            self.files.insert(path.to_string(), content.to_vec());
        }

        pub fn file(&self, path: &str) -> Option<&[u8]> {
            self.files.get(path).map(Vec::as_slice)
        }

        pub fn set_perms(&mut self, path: &str, permissions: u32) {
            self.perms.insert(path.to_string(), permissions);
        }
    }

    impl HostApi for MockHost {
        fn get_server(&mut self, id: u64) -> HostResult<Option<ServerInfo>> {
            Ok(self.servers.get(&id).cloned())
        }

        fn get_game(&mut self, code: &str) -> HostResult<Option<GameInfo>> {
            Ok(self.games.get(code).cloned())
        }

        fn get_node(&mut self, id: u64) -> HostResult<Option<NodeInfo>> {
            Ok(self.nodes.get(&id).cloned())
        }

        fn read_dir(&mut self, _node_id: u64, path: &str) -> HostResult<Option<Vec<DirEntry>>> {
            let path = path.trim_end_matches('/');
            if !self.dirs.contains(path) {
                return Ok(None);
            }
            let prefix = format!("{path}/");
            let mut entries = Vec::new();
            for dir in &self.dirs {
                if let Some(rest) = dir.strip_prefix(&prefix)
                    && !rest.is_empty()
                    && !rest.contains('/')
                {
                    entries.push(DirEntry {
                        name: rest.to_string(),
                        is_dir: true,
                    });
                }
            }
            for file in self.files.keys() {
                if let Some(rest) = file.strip_prefix(&prefix)
                    && !rest.is_empty()
                    && !rest.contains('/')
                {
                    entries.push(DirEntry {
                        name: rest.to_string(),
                        is_dir: false,
                    });
                }
            }
            Ok(Some(entries))
        }

        fn stat(&mut self, _node_id: u64, path: &str) -> HostResult<Option<FileStat>> {
            let path = path.trim_end_matches('/');
            if let Some(content) = self.files.get(path) {
                return Ok(Some(FileStat {
                    is_dir: false,
                    size: content.len() as u64,
                    permissions: self.perms.get(path).copied().unwrap_or(0o644),
                }));
            }
            if self.dirs.contains(path) {
                return Ok(Some(FileStat {
                    is_dir: true,
                    size: 0,
                    permissions: 0o755,
                }));
            }
            Ok(None)
        }

        fn download(&mut self, _node_id: u64, path: &str) -> HostResult<Vec<u8>> {
            self.files
                .get(path)
                .cloned()
                .ok_or_else(|| HostApiError::Op(format!("no such file: {path}")))
        }

        fn upload(
            &mut self,
            _node_id: u64,
            path: &str,
            content: &[u8],
            _permissions: u32,
        ) -> HostResult<()> {
            self.add_file(path, content);
            Ok(())
        }

        fn remove(&mut self, _node_id: u64, path: &str, _recursive: bool) -> HostResult<()> {
            if self.files.remove(path).is_none() {
                return Err(HostApiError::Op(format!("no such file: {path}")));
            }
            Ok(())
        }

        fn chmod(&mut self, _node_id: u64, path: &str, permissions: u32) -> HostResult<()> {
            if !self.files.contains_key(path) && !self.dirs.contains(path) {
                return Err(HostApiError::Op(format!("no such file: {path}")));
            }
            self.chmods.push((path.to_string(), permissions));
            self.perms.insert(path.to_string(), permissions);
            Ok(())
        }

        fn execute_command(
            &mut self,
            _node_id: u64,
            command: &str,
            work_dir: Option<&str>,
        ) -> HostResult<CommandOutput> {
            self.commands
                .push((command.to_string(), work_dir.map(str::to_string)));
            Ok(self.command_results.pop_front().unwrap_or(CommandOutput {
                output: String::new(),
                exit_code: 0,
            }))
        }

        fn log_info(&mut self, message: &str) {
            self.logs.push(format!("INFO {message}"));
        }

        fn log_error(&mut self, message: &str) {
            self.logs.push(format!("ERROR {message}"));
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use gameap_plugin_sdk::host;
    use gameap_plugin_sdk::proto::gameap::plugin::sdk::{games, nodecmd, nodefs, nodes, servers};

    use super::*;

    fn call_err(err: gameap_plugin_sdk::HostError) -> HostApiError {
        HostApiError::Call(err.to_string())
    }

    impl HostApi for WasmHost {
        fn get_server(&mut self, id: u64) -> HostResult<Option<ServerInfo>> {
            let resp = host::servers::get_server(&servers::GetServerRequest { id })
                .map_err(call_err)?;
            Ok(resp
                .found
                .then_some(resp.server)
                .flatten()
                .map(|s| ServerInfo {
                    id: s.id,
                    game_code: s.game_id,
                    node_id: s.ds_id,
                    dir: s.dir,
                }))
        }

        fn get_game(&mut self, code: &str) -> HostResult<Option<GameInfo>> {
            let resp = host::games::get_game(&games::GetGameRequest {
                code: code.to_owned(),
            })
            .map_err(call_err)?;
            Ok(resp.found.then_some(resp.game).flatten().map(|g| GameInfo {
                code: g.code,
                name: g.name,
                engine: g.engine,
            }))
        }

        fn get_node(&mut self, id: u64) -> HostResult<Option<NodeInfo>> {
            let resp = host::nodes::get_node(&nodes::GetNodeRequest { id }).map_err(call_err)?;
            Ok(resp.found.then_some(resp.node).flatten().map(|n| NodeInfo {
                id: n.id,
                os: n.os,
                work_path: n.work_path,
            }))
        }

        fn read_dir(&mut self, node_id: u64, path: &str) -> HostResult<Option<Vec<DirEntry>>> {
            let resp = host::nodefs::read_dir(&nodefs::ReadDirRequest {
                node_id,
                path: path.to_owned(),
            })
            .map_err(call_err)?;
            if resp.error.is_some() {
                return Ok(None);
            }
            let dir_type = nodefs::FileType::Dir as i32;
            Ok(Some(
                resp.files
                    .into_iter()
                    .map(|f| DirEntry {
                        name: f.name,
                        is_dir: f.r#type == dir_type,
                    })
                    .collect(),
            ))
        }

        fn stat(&mut self, node_id: u64, path: &str) -> HostResult<Option<FileStat>> {
            let resp = host::nodefs::get_file_info(&nodefs::GetFileInfoRequest {
                node_id,
                path: path.to_owned(),
            })
            .map_err(call_err)?;
            let dir_type = nodefs::FileType::Dir as i32;
            Ok(resp.found.then_some(resp.file).flatten().map(|f| FileStat {
                is_dir: f.r#type == dir_type,
                size: f.size,
                permissions: f.permissions,
            }))
        }

        fn download(&mut self, node_id: u64, path: &str) -> HostResult<Vec<u8>> {
            let resp = host::nodefs::download(&nodefs::DownloadRequest {
                node_id,
                path: path.to_owned(),
            })
            .map_err(call_err)?;
            match resp.error {
                Some(err) => Err(HostApiError::Op(err)),
                None => Ok(resp.content),
            }
        }

        fn upload(
            &mut self,
            node_id: u64,
            path: &str,
            content: &[u8],
            permissions: u32,
        ) -> HostResult<()> {
            let resp = host::nodefs::upload(&nodefs::UploadRequest {
                node_id,
                path: path.to_owned(),
                content: content.to_vec(),
                permissions,
            })
            .map_err(call_err)?;
            if resp.success {
                Ok(())
            } else {
                Err(HostApiError::Op(resp.error.unwrap_or_default()))
            }
        }

        fn remove(&mut self, node_id: u64, path: &str, recursive: bool) -> HostResult<()> {
            let resp = host::nodefs::remove(&nodefs::RemoveRequest {
                node_id,
                path: path.to_owned(),
                recursive,
            })
            .map_err(call_err)?;
            if resp.success {
                Ok(())
            } else {
                Err(HostApiError::Op(resp.error.unwrap_or_default()))
            }
        }

        fn chmod(&mut self, node_id: u64, path: &str, permissions: u32) -> HostResult<()> {
            let resp = host::nodefs::chmod(&nodefs::ChmodRequest {
                node_id,
                path: path.to_owned(),
                permissions,
            })
            .map_err(call_err)?;
            if resp.success {
                Ok(())
            } else {
                Err(HostApiError::Op(resp.error.unwrap_or_default()))
            }
        }

        fn execute_command(
            &mut self,
            node_id: u64,
            command: &str,
            work_dir: Option<&str>,
        ) -> HostResult<CommandOutput> {
            let resp = host::nodecmd::execute_command(&nodecmd::ExecuteCommandRequest {
                node_id,
                command: command.to_owned(),
                work_dir: work_dir.map(str::to_owned),
            })
            .map_err(call_err)?;
            match resp.error {
                Some(err) => Err(HostApiError::Op(err)),
                None => Ok(CommandOutput {
                    output: resp.output,
                    exit_code: resp.exit_code,
                }),
            }
        }

        fn log_info(&mut self, message: &str) {
            host::log::info(message);
        }

        fn log_error(&mut self, message: &str) {
            host::log::error(message);
        }
    }
}
