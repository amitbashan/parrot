use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};

use crate::document::Document;

pub struct Project {
    path: PathBuf,
    open_documents: HashMap<PathBuf, Document>,
}

impl Project {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    fn open_document_if_not_loaded<P: AsRef<Path>>(&mut self, relative_path: P) -> io::Result<()> {
        if !self.open_documents.contains_key(relative_path.as_ref()) {
            let path = self.path.join(&relative_path);
            let text = fs::read_to_string(path)?;
            let document = Document::from(text);
            self.open_documents
                .insert(relative_path.as_ref().to_path_buf(), document);
        }
        Ok(())
    }

    pub fn open_document<P: AsRef<Path>>(&mut self, relative_path: P) -> io::Result<&Document> {
        self.open_document_if_not_loaded(&relative_path)?;
        Ok(&self.open_documents[relative_path.as_ref()])
    }

    pub fn open_document_mut<P: AsRef<Path>>(
        &mut self,
        relative_path: P,
    ) -> io::Result<&mut Document> {
        self.open_document_if_not_loaded(&relative_path)?;
        Ok(self.open_documents.get_mut(relative_path.as_ref()).unwrap())
    }
}

impl From<PathBuf> for Project {
    fn from(value: PathBuf) -> Self {
        Self {
            path: value,
            open_documents: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tree(DiGraph<PathBuf, ()>);

impl Tree {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let mut dirs = vec![(path.as_ref().to_path_buf(), fs::read_dir(&path)?)];
        let mut tree = DiGraph::new();

        while let Some((path, dir)) = dirs.pop() {
            let parent = tree.add_node(path);
            for entry in dir {
                let entry = entry?;
                let child = tree.add_node(entry.path());
                tree.add_edge(parent, child, ());
            }
        }

        Ok(Self(tree))
    }
}
