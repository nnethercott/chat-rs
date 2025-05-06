use crate::models::qwen::{Model as Qwen, generate};
use std::{env, sync::Arc};

const MODEL_WORKER_THREADS: usize = 1;

//TODO use ssomething like clap for extracting from env

fn spawn_workers() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let rx = Arc::new(rx);

    let mut handles = vec![];

    for _ in 0..env::var("MODEL_WORKER_THREADS")
        .map(|v| usize::from_str_radix(&v, 10))
        .unwrap()
        .unwrap()
    {
        let rx_worker = Arc::clone(&rx);
        let mut model = Qwen::from_pretrained("Qwen/Qwen2-0.5B".into()).unwrap();

        let handle = std::thread::spawn(move || {
            loop {
                while let Ok(message) = rx_worker.recv() {
                    generate(&mut model, message, 32, todo!());
                }
            }
        });

        handles.push(handle);
    }

    for h in handles {
        h.join();
    }
}
