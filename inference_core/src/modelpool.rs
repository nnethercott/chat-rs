use futures::channel::oneshot;
use tracing::{debug, error, info};

use crate::errors::{Error, Result};
use crate::models::qwen::{Model as Qwen, generate};
use std::thread;
use std::{env, sync::Arc, thread::JoinHandle};

const MODEL_WORKER_THREADS: usize = 1;

// TODO: make Message enum for internal usage; streaming and blob
pub struct StreamBackMessage {
    pub prompt: String,
    pub sender: Option<tokio::sync::mpsc::Sender<String>>,
}

#[derive(Debug)]
pub struct ModelPool {
    workers: Arc<Vec<JoinHandle<Result<()>>>>,
    pub tx: Option<crossbeam_channel::Sender<StreamBackMessage>>,
}

impl ModelPool {
    pub fn infer(&self, request: StreamBackMessage) -> Result<()> {
        match &self.tx {
            Some(t) => Ok(t.send(request)?),
            None => {
                error!("sender dropped");
                Err(Error::Other {
                    reason: "sender already dropped",
                })
            }
        }
    }

    pub fn spawn(num_replicas: usize) -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let rx = Arc::new(rx);

        info!(tokio_handle=?tokio::runtime::Handle::try_current());

        let mut workers = vec![];

        for _ in 0..num_replicas {
            let rx_worker = Arc::clone(&rx);

            let handle = std::thread::spawn(move || {
                let mut model = Qwen::from_pretrained("Qwen/Qwen2-0.5B".into())
                    .map_err(Error::ModelLoadError)?;

                info!("model loaded in thread: {:?}", thread::current().id());

                loop {
                    while let Ok(msg) = rx_worker.recv() {
                        let StreamBackMessage {
                            prompt,
                            sender: maybe_sender,
                        } = msg;

                        if let Err(e) = generate(&mut model, prompt, 32, maybe_sender){
                            error!(error=?e);
                        }
                    }
                }
            });

            workers.push(handle);
        }

        Ok(Self {
            workers: Arc::new(workers),
            tx: Some(tx),
        })
    }
}

impl Drop for ModelPool {
    fn drop(&mut self) {
        drop(self.tx.take());

        if let Some(workers) = Arc::get_mut(&mut self.workers) {
            // kinda like into_iter
            for w in workers.drain(..) {
                w.join();
            }
        }
    }
}
