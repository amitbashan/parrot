use std::{
    collections::{HashMap, HashSet},
    io,
    net::SocketAddr,
};

use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::{
    client::{request, Request},
    project::{Project, Tree},
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
        Notify(notify::Notification),
    }

    pub mod notify {
        use crate::client::request::update::Commit;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, Clone)]
        pub enum Notification {
            Commit(Commit),
        }
    }
}

pub struct Server {
    listener: TcpListener,
    project: Project,
    clients: HashMap<SocketAddr, Framed<TcpStream, LengthDelimitedCodec>>,
}

impl Server {
    pub async fn bind<A: ToSocketAddrs>(address: A, project: Project) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(address).await?,
            project,
            clients: Default::default(),
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

    async fn accept(&mut self) -> anyhow::Result<()> {
        let (stream, address) = self.listener.accept().await?;
        log::info!("Accepting connection from: {address}");
        let stream = self.register_client(stream, address);
        let request = stream.next().await.unwrap()?;
        let request: Request = flexbuffers::from_slice(&request)?;
        let response = self.handle_request(address, request).await?;
        self.respond(address, response).await?;
        Ok(())
    }

    async fn handle_request(
        &mut self,
        address: SocketAddr,
        request: Request,
    ) -> anyhow::Result<Response> {
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
                    self.commit(address, commit).await?;
                    Ok(Response::Acknowledge)
                }
            },
        }
    }

    fn register_client(
        &mut self,
        stream: TcpStream,
        address: SocketAddr,
    ) -> &mut Framed<TcpStream, LengthDelimitedCodec> {
        if !self.clients.contains_key(&address) {
            log::info!("Registering new client from: {address}");
            self.clients
                .insert(address, Framed::new(stream, LengthDelimitedCodec::new()));
        }
        self.get_client_mut(address)
    }

    fn get_client_mut(
        &mut self,
        address: SocketAddr,
    ) -> &mut Framed<TcpStream, LengthDelimitedCodec> {
        self.clients.get_mut(&address).unwrap()
    }

    async fn respond(&mut self, address: SocketAddr, response: Response) -> anyhow::Result<()> {
        let stream = self.get_client_mut(address);
        let response = flexbuffers::to_vec(response)?;
        stream.send(response.into()).await?;
        Ok(())
    }

    async fn commit(
        &mut self,
        address: SocketAddr,
        commit: request::update::Commit,
    ) -> anyhow::Result<()> {
        let request::update::Commit {
            ref document_path,
            ref insertions,
            ref deletions,
        } = commit;
        let document = self.project.open_document_mut(document_path)?;

        for range in deletions {
            document.remove(range.clone());
        }

        for (index, text) in insertions {
            document.insert(*index, text.as_str());
        }

        self.notify(address, &response::notify::Notification::Commit(commit))
            .await?;

        Ok(())
    }

    async fn notify(
        &mut self,
        notifier: SocketAddr,
        notification: &response::notify::Notification,
    ) -> anyhow::Result<()> {
        let clients = self
            .clients
            .keys()
            .filter(|client| client != &&notifier)
            .copied()
            .collect::<HashSet<_>>();
        for client in clients {
            self.respond(client, Response::Notify(notification.clone()))
                .await?;
        }

        Ok(())
    }
}
