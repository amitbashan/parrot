use std::io;

use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    client::{request, Request},
    project::{ServerProject, Tree},
};

pub use response::Response;

pub mod response {
    use serde::{Deserialize, Serialize};

    use crate::{document::Document, project::Tree};

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Response {
        Acknowledge,
        Document(Document),
        ProjectTree(Tree),
    }
}

pub struct Server {
    listener: TcpListener,
    project: ServerProject,
}

impl Server {
    pub async fn bind<A: ToSocketAddrs>(address: A, project: ServerProject) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(address).await?,
            project,
        })
    }

    pub async fn start(&mut self) -> io::Result<()> {
        log::info!(
            "Server started, listening on {}. Project path: {}",
            self.listener.local_addr()?,
            self.project.path().display(),
        );
        loop {
            if let Err(e) = self.accept().await {
                log::error!("{e}");
            }
        }
    }

    pub async fn accept(&mut self) -> anyhow::Result<()> {
        let (stream, address) = self.listener.accept().await?;
        log::info!("Accepting connection: {address}");
        let mut stream = Framed::new(stream, LengthDelimitedCodec::new());
        let request = stream.next().await.unwrap()?;
        let request: Request = flexbuffers::from_slice(&request)?;
        let response = self.handle_request(request)?;
        let response = flexbuffers::to_vec(response)?;
        stream.send(response.into()).await?;
        Ok(())
    }

    fn handle_request(&mut self, request: Request) -> anyhow::Result<Response> {
        match request {
            Request::Fetch(fetch) => match fetch {
                request::Fetch::ProjectTree => {
                    let tree = Tree::new(self.project.path())?;
                    Ok(Response::ProjectTree(tree))
                }
                request::Fetch::Document(path) => {
                    let document = self.project.open_document(path)?;
                    Ok(Response::Document(document.clone()))
                }
            },
            Request::Update(update) => match update {
                request::Update::Commit(commit) => {
                    self.commit(commit)?;
                    Ok(Response::Acknowledge)
                }
            },
        }
    }

    fn commit(&mut self, commit: request::update::Commit) -> io::Result<()> {
        let request::update::Commit {
            document_path,
            insertions,
            deletions,
        } = commit;
        let document = self.project.open_document_mut(&document_path)?;

        for range in deletions {
            document.remove(range);
        }

        for (index, text) in insertions {
            document.insert(index, &text);
        }

        Ok(())
    }
}
