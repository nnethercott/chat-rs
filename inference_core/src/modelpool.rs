use tracing::{error, info};

use crate::errors::{Error, Result};
use crate::models::qwen::{Model as Qwen, generate};
use std::thread;
use std::{sync::Arc, thread::JoinHandle};

// const MODEL_WORKER_THREADS: usize = 1;

// TODO: make Message enum for internal usage; streaming and blob
pub enum SendBackMessage {
    Streaming {
        prompt: String,
        sender: tokio::sync::mpsc::Sender<String>,
        opts: Opts,
    },
    Blocking {
        prompt: String,
        sender: tokio::sync::oneshot::Sender<String>,
        opts: Opts,
    },
}

// options to control generation
pub struct Opts{
    pub max_new_tokens: usize, 
    pub eos_tokens: Vec<&'static str>,
    pub top_k: Option<usize>,
    pub top_p: Option<f64>,
    pub temperature: Option<f64>,
    pub repeat_penalty: Option<f32>,
}

impl Default for Opts{
    fn default() -> Self {
        Self{
            max_new_tokens: 128, 
            eos_tokens: vec![],
            temperature: Some(0.8),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct ModelPool {
    workers: Arc<Vec<JoinHandle<Result<()>>>>,
    pub tx: Option<crossbeam_channel::Sender<SendBackMessage>>,
}

impl ModelPool {
    pub fn infer(&self, request: SendBackMessage) -> Result<()> {
        match &self.tx {
            Some(t) => Ok(t.send(request)?),
            None => Err(Error::Other {
                reason: "sender already dropped",
            }),
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
                let mut model = Qwen::from_pretrained("Qwen/Qwen2.5-0.5B-Instruct".into())
                    .map_err(Error::ModelLoadError)?;

                let t_id = thread::current().id();
                info!("model loaded in thread: {:?}", t_id);

                loop {
                    while let Ok(msg) = rx_worker.recv() {
                        info!("request was picked up in thread: {:?}", t_id);

                        match msg {
                            SendBackMessage::Streaming { prompt, sender, opts } => {
                                if let Err(e) = generate(&mut model, prompt, opts, Some(sender)) {
                                    error!(error=?e);
                                }
                            }
                            SendBackMessage::Blocking { prompt, sender, opts } => {
                                match generate(&mut model, prompt, opts, None) {
                                    Ok(resp) => sender.send(resp).expect("failed to send back"),
                                    Err(e) => error!(error=?e),
                                }
                            }
                        };

                        model.inner.clear_kv_cache();
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
                let _ = w.join();
            }
        }
    }
}
