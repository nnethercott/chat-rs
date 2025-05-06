use futures::channel::oneshot;
use tracing::{error, info};

use crate::errors::{Error, Result};
use crate::models::qwen::{Model as Qwen, generate};
use std::thread;
use std::{env, sync::Arc, thread::JoinHandle};

const MODEL_WORKER_THREADS: usize = 1;

pub struct StreamBackMessage {
    pub prompt: String,
    pub sender: tokio::sync::mpsc::Sender<u32>, // stream tokens back
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

        let mut workers = vec![];

        for _ in 0..num_replicas {
            let rx_worker = Arc::clone(&rx);

            let handle = std::thread::spawn(move || {
                let mut model = Qwen::from_pretrained("Qwen/Qwen2-0.5B".into())
                    .map_err(Error::ModelLoadError)?;

                info!("model loaded in thread: {:?}", thread::current().id());

                loop {
                    while let Ok(StreamBackMessage { prompt, sender }) = rx_worker.recv() {
                        generate(&mut model, prompt, 32, sender);
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
