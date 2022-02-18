use super::project_watcher::ProjectWatcher;
use crate::build_backend::build_backend;
use crate::config::Config;
use crate::run_backend::run_backend;
use crate::BuildMode;
use anyhow::{Context, Error, Result};
use fehler::throws;
use parking_lot::Mutex;
use std::sync::Arc;
use tokio::process::Child;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::{spawn, task::JoinHandle, time::Duration};

pub struct BackendWatcher {
    watcher: ProjectWatcher,
    task: JoinHandle<Result<()>>,
}

impl BackendWatcher {
    #[throws]
    pub async fn start(
        config: &Config,
        build_mode: BuildMode,
        debounce_time: Duration,
        server: Arc<Mutex<Option<Child>>>,
    ) -> Self {
        let (watcher, debounced_receiver) =
            ProjectWatcher::start(&config.watch.backend, debounce_time)
                .context("Failed to start the backend project watcher")?;
        Self {
            watcher,
            task: spawn(on_change(
                debounced_receiver,
                build_mode,
                config.https,
                server,
            )),
        }
    }

    #[throws]
    pub async fn stop(self) {
        self.watcher.stop().await?;
        self.task.await??;
    }
}

#[throws]
async fn on_change(
    mut receiver: UnboundedReceiver<()>,
    build_mode: BuildMode,
    https: bool,
    server: Arc<Mutex<Option<Child>>>,
) {
    let mut build_task = None::<JoinHandle<()>>;

    while receiver.recv().await.is_some() {
        if let Some(build_task) = build_task.take() {
            build_task.abort();
        }

        let server_process = { server.lock().take() };
        if let Some(mut server) = server_process {
            let _ = server.kill().await;
        }

        build_task = Some(spawn(build_and_run(Arc::clone(&server), build_mode, https)));
    }

    if let Some(build_task) = build_task.take() {
        build_task.abort();
    }
}

async fn build_and_run(server: Arc<Mutex<Option<Child>>>, build_mode: BuildMode, https: bool) {
    if let Err(error) = build_backend(build_mode, https).await {
        return eprintln!("{}", error);
    }
    match run_backend(build_mode) {
        Ok(backend) => *server.lock() = Some(backend),
        Err(error) => eprintln!("{}", error),
    }
}
