use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::thread;
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use futures_util::{Stream, stream};

use crate::paths;

static IPC_RECEIVER: OnceLock<async_channel::Receiver<IpcEnvelope>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum IpcCommand {
    Open { panel: String },
    Close,
    Dash { tab: Option<String> },
    Lock,
    Tray { args: Vec<String> },
    Settings { args: Vec<String> },
    Theme { args: Vec<String> },
    Weather,
    Clipboard { args: Vec<String> },
    ClipboardPush { content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcRequest {
    pub command: IpcCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct IpcEnvelope {
    pub request: IpcRequest,
    pub reply: async_channel::Sender<IpcResponse>,
}

pub fn set_subscription_receiver(receiver: async_channel::Receiver<IpcEnvelope>) {
    let _ = IPC_RECEIVER.set(receiver);
}

pub fn subscribe() -> impl Stream<Item = IpcEnvelope> + Send + 'static {
    let receiver = IPC_RECEIVER
        .get()
        .expect("CDS IPC receiver must be registered before starting the Iced daemon")
        .clone();

    stream::unfold(receiver, |receiver| async move {
        receiver
            .recv()
            .await
            .ok()
            .map(|envelope| (envelope, receiver))
    })
}

pub struct SocketGuard {
    path: PathBuf,
}

impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn start_server() -> Result<(async_channel::Receiver<IpcEnvelope>, SocketGuard)> {
    let path = paths::socket_path()?;

    if UnixStream::connect(&path).is_ok() {
        bail!("CDS is already running ({} is accepting connections)", path.display());
    }

    if path.exists() {
        std::fs::remove_file(&path)
            .with_context(|| format!("could not remove stale socket {}", path.display()))?;
    }

    let listener = UnixListener::bind(&path)
        .with_context(|| format!("could not create IPC socket {}", path.display()))?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;

    let (sender, receiver) = async_channel::unbounded::<IpcEnvelope>();
    thread::Builder::new()
        .name("cds-ipc".into())
        .spawn(move || {
            for incoming in listener.incoming() {
                match incoming {
                    Ok(stream) => {
                        let sender = sender.clone();
                        let _ = thread::Builder::new()
                            .name("cds-ipc-client".into())
                            .spawn(move || handle_client(stream, sender));
                    }
                    Err(error) => {
                        tracing::error!(%error, "CDS IPC listener failed");
                        break;
                    }
                }
            }
        })
        .context("could not start the CDS IPC listener")?;

    Ok((receiver, SocketGuard { path }))
}

fn handle_client(mut stream: UnixStream, sender: async_channel::Sender<IpcEnvelope>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

    let Ok(reader_stream) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();

    let response = if reader.read_line(&mut line).is_ok() {
        match serde_json::from_str::<IpcRequest>(&line) {
            Ok(request) => {
                let (reply, receiver) = async_channel::bounded(1);
                if sender.send_blocking(IpcEnvelope { request, reply }).is_ok() {
                    receiver.recv_blocking().unwrap_or_else(|_| IpcResponse {
                        ok: false,
                        message: "CDS did not return an IPC response".into(),
                    })
                } else {
                    IpcResponse {
                        ok: false,
                        message: "CDS IPC queue is unavailable".into(),
                    }
                }
            }
            Err(error) => IpcResponse {
                ok: false,
                message: format!("invalid IPC request: {error}"),
            },
        }
    } else {
        IpcResponse {
            ok: false,
            message: "could not read IPC request".into(),
        }
    };

    if let Ok(mut payload) = serde_json::to_vec(&response) {
        payload.push(b'\n');
        let _ = stream.write_all(&payload);
        let _ = stream.flush();
    }
}

pub fn send(command: IpcCommand) -> Result<IpcResponse> {
    let path = paths::socket_path()?;
    let mut stream = UnixStream::connect(&path)
        .with_context(|| format!("could not connect to CDS at {}; is the shell running?", path.display()))?;
    stream.set_read_timeout(Some(Duration::from_secs(6)))?;

    let request = IpcRequest { command };
    let mut payload = serde_json::to_vec(&request)?;
    payload.push(b'\n');
    stream.write_all(&payload)?;
    stream.flush()?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).context("CDS closed the IPC connection without replying")?;
    serde_json::from_str(&line).context("CDS returned an invalid IPC response")
}

pub fn is_running() -> bool {
    paths::socket_path()
        .ok()
        .is_some_and(|path| UnixStream::connect(path).is_ok())
}
