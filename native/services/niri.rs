use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use futures_util::{Stream, stream};
use niri_ipc::{
    Action, Request, Response, WorkspaceReferenceArg,
    socket::Socket,
};

pub fn subscribe() -> impl Stream<Item = niri_ipc::Event> + Send + 'static {
    let (sender, receiver) = async_channel::unbounded();

    let spawn_result = thread::Builder::new()
        .name("cds-niri-events".into())
        .spawn(move || loop {
            match Socket::connect() {
                Ok(mut socket) => match socket.send(Request::EventStream) {
                    Ok(Ok(Response::Handled)) => {
                        let mut read_event = socket.read_events();

                        loop {
                            match read_event() {
                                Ok(event) => {
                                    if sender.send_blocking(event).is_err() {
                                        return;
                                    }
                                }
                                Err(error) => {
                                    tracing::warn!(%error, "Niri event stream disconnected");
                                    break;
                                }
                            }
                        }
                    }
                    Ok(Ok(response)) => {
                        tracing::warn!(?response, "Niri did not start the event stream");
                    }
                    Ok(Err(error)) => {
                        tracing::warn!(%error, "Niri rejected the event stream request");
                    }
                    Err(error) => {
                        tracing::warn!(%error, "Could not request Niri event stream");
                    }
                },
                Err(error) => {
                    tracing::debug!(%error, "Niri IPC is not available yet");
                }
            }

            if sender.is_closed() {
                return;
            }
            thread::sleep(Duration::from_secs(2));
        });

    if let Err(error) = spawn_result {
        tracing::error!(%error, "Could not start Niri event worker");
    }

    stream::unfold(receiver, |receiver| async move {
        receiver
            .recv()
            .await
            .ok()
            .map(|event| (event, receiver))
    })
}

pub fn focus_workspace(index: u8) {
    let _ = thread::Builder::new()
        .name("cds-niri-action".into())
        .spawn(move || {
            if let Err(error) = focus_workspace_blocking(index) {
                tracing::warn!(%error, workspace = index, "Could not focus Niri workspace");
            }
        });
}

fn focus_workspace_blocking(index: u8) -> Result<()> {
    let mut socket = Socket::connect().context("could not connect to Niri IPC")?;
    match socket
        .send(Request::Action(Action::FocusWorkspace {
            reference: WorkspaceReferenceArg::Index(index),
        }))
        .context("could not send workspace focus action")?
    {
        Ok(Response::Handled) => Ok(()),
        Ok(response) => bail!("unexpected Niri response: {response:?}"),
        Err(error) => bail!("Niri rejected workspace focus: {error}"),
    }
}
