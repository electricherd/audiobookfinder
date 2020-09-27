//! Wraps bktree functionality, the container and defines audio info structs to be used
use super::bktree::{BKTree, Distance};
use std::{boxed::Box, time::Duration, vec::Vec};

/// The container keeps the collection data. It currently consists of a BKTree
/// (https://en.wikipedia.org/wiki/BK-tree), because key is a string of lexical
/// data.
pub struct Container {
    bk_tree: BKTree<AudioInfoKey, Box<AudioInfo>>,
}
impl Container {
    pub fn new() -> Self {
        Self {
            bk_tree: BKTree::new(),
        }
    }
    /// wraps and protects the bktree find but with AudioInfoKey
    pub fn find(
        &self,
        searcher: &AudioInfoKey,
        tolerance: usize,
    ) -> (Vec<&Box<AudioInfo>>, Vec<&AudioInfoKey>) {
        self.bk_tree.find(&searcher, tolerance)
    }

    /// wraps and protects the bktree insert but with AudioInfoKey
    pub fn insert(&mut self, key: AudioInfoKey, value: Box<AudioInfo>) {
        self.bk_tree.insert(key, value);
    }

    pub fn flush(&self) -> Vec<(&AudioInfoKey, &Box<AudioInfo>)> {
        self.bk_tree.dfs()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioInfoKey {
    k: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioInfo {
    pub duration: Duration,
    pub album: String,
    pub file_name: String,
    // todo: more information should be used
}

/// protect handling of AudioInfoKey
impl AudioInfoKey {
    /// Creates from the given input a key!
    /// The idea is to get a as unique key to possibly identify
    /// duplicates, but to give enough distance (hence a string
    /// distance metric is used) to similar but other keys.
    pub fn new(artist: &String, title: &String) -> Self {
        Self {
            k: format!("{} {}", artist, title),
        }
    }
    pub fn get(&self) -> &String {
        &self.k
    }
}

impl Distance for AudioInfoKey {
    fn distance(&self, other: &Self) -> usize {
        super::bktree::osa_distance(&self.k.clone(), &other.k)
    }
}
