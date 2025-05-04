use async_trait::async_trait;
use hf_hub::{
    Repo, RepoType,
    api::tokio::{Api, ApiRepo},
};
use std::ops::Deref;
use tokio::sync::mpsc::Sender;

pub type Tokens = Vec<u32>;

pub struct HfApiManager {
    repo: ApiRepo,
}

impl HfApiManager {
    pub fn new(repo_id: String) -> anyhow::Result<Self> {
        let api = Api::new()?;
        let repo = api.repo(Repo::new(repo_id, RepoType::Model));
        Ok(Self { repo })
    }
}
impl Deref for HfApiManager {
    type Target = ApiRepo;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

// impl GenerativeModel<T: Model>
#[async_trait]
pub trait GenerativeModel {
    async fn generate_stream(&mut self, prompt: String, tx: Sender<u32>) -> anyhow::Result<()>;
}

pub struct TempModel;

#[async_trait]
impl GenerativeModel for TempModel {
    async fn generate_stream(&mut self, prompt: String, tx: Sender<u32>) ->anyhow::Result<()>{
        for i in 0..10 {
            tx.send(i).await.ok();
        }
        Ok(())
    }
}
