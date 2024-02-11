use std::io;

use futures::{SinkExt, StreamExt};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

use crate::server::Response;

pub use request::Request;

pub mod request {
    use std::path::PathBuf;

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Request {
        Fetch(Fetch),
        Update(Update),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Fetch {
        ProjectTree,
        Document(PathBuf),
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Update {
        Commit(update::Commit),
    }

    pub mod update {
        use std::{collections::HashMap, path::PathBuf};

        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, Clone)]
        pub struct Commit {
            pub document_path: PathBuf,
            pub insertions: HashMap<usize, String>,
            pub deletions: Vec<std::ops::Range<usize>>,
        }
    }
}

pub struct Client {
    stream: Framed<TcpStream, LengthDelimitedCodec>,
}

impl Client {
    pub async fn new<A: ToSocketAddrs>(server_address: A) -> io::Result<Self> {
        Ok(Self {
            stream: Framed::new(
                TcpStream::connect(server_address).await?,
                LengthDelimitedCodec::new(),
            ),
        })
    }

    pub async fn request(&mut self, request: Request) -> anyhow::Result<Response> {
        let request = flexbuffers::to_vec(request)?;
        self.stream.send(request.into()).await?;
        let response = self.stream.next().await.unwrap()?;
        let response: Response = flexbuffers::from_slice(&response)?;
        Ok(response)
    }
}
