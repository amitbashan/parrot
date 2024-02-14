use core::fmt;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};

use crate::document::Document;

#[derive(Debug)]
pub enum ProjectError {
    OutOfBounds,
}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::OutOfBounds => "Object requested by client is out of the project's bounds",
            }
        )
    }
}

impl std::error::Error for ProjectError {}

pub struct Project {
    path: PathBuf,
    open_documents: HashMap<PathBuf, Document>,
}

impl Project {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    fn is_valid_path<P: AsRef<Path>>(&self, path: P) -> bool {
        if let Ok(resolved_path) = fs::canonicalize(path) {
            resolved_path.starts_with(&self.path)
        } else {
            false
        }
    }

    fn open_document_if_not_loaded<P: AsRef<Path>>(
        &mut self,
        relative_path: P,
    ) -> anyhow::Result<()> {
        if !self.is_valid_path(self.path.join(&relative_path)) {
            return Err(ProjectError::OutOfBounds.into());
        }

        if !self.open_documents.contains_key(relative_path.as_ref()) {
            let path = self.path.join(&relative_path);
            let text = fs::read_to_string(path)?;
            let document = Document::from(text);
            self.open_documents
                .insert(relative_path.as_ref().to_path_buf(), document);
        }
        Ok(())
    }

    pub fn open_document<P: AsRef<Path>>(&mut self, relative_path: P) -> anyhow::Result<&Document> {
        self.open_document_if_not_loaded(&relative_path)?;
        Ok(&self.open_documents[relative_path.as_ref()])
    }

    pub fn open_document_mut<P: AsRef<Path>>(
        &mut self,
        relative_path: P,
    ) -> anyhow::Result<&mut Document> {
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
