// MIT License
//
// Copyright (c) 2020 Ben
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// © 2020 GitHub, Inc.
// Terms
// https://github.com/tempor1s/bktree-rs
use std::{char, cmp::min, collections::HashMap};

/// BKTree structure that is used to store word like structures
/// and perform "fuzzy" search on them to implement "do you mean"
/// functionality on them. Can perform said search on any term that implements
/// the distance trait. The default implementation is Osa distance.
#[derive(Default)]
pub struct BKTree<K, V>
where
    K: Distance,
{
    root: Option<BKTreeNode<K, V>>,
}

impl<K, V> BKTree<K, V>
where
    K: Distance,
{
    /// Create a new BK Tree with an empty root.
    pub fn new() -> BKTree<K, V> {
        BKTree { root: None }
    }

    /// Create a new tree from the items in a Vector.
    /// Useful for inserting a lot of items from a file etc.
    /// Vector requires a Vec of tuples of K, V pairs where K implements Distance trait
    #[allow(dead_code)]
    pub fn new_from_vec(items: Vec<(K, V)>) -> BKTree<K, V> {
        let mut tree = BKTree { root: None };

        for item in items {
            tree.insert(item.0, item.1);
        }

        return tree;
    }

    /// Add a new (key, value) pair into the BKTree.
    pub fn insert(&mut self, key: K, value: V) {
        // If the root exists, insert from there.
        if let Some(root) = &mut self.root {
            root.insert(key, value);
        } else {
            // otherwise, set the root to be a new BKTreeNode
            self.root = Some(BKTreeNode::new(key, value));
        }
    }

    /// Search for the closest Item to the key with a given tolerence. (Steps to get there)
    /// The return value is a tuple of 2 Vecs, where the first one has exact matches and the second
    /// are matches within the given tolerence.
    ///
    /// A match is approximate if the distance between key1 and key2 are less than the given tolerence.
    pub fn find(&self, key: &K, tolerence: usize) -> (Vec<&V>, Vec<&K>) {
        // if our root exists, search from the root
        return if let Some(root) = &self.root {
            root.find(&key, tolerence)
        } else {
            // if we can not find anything, return a tuple of empty vectors
            (vec![], vec![])
        };
    }

    pub fn dfs(&self) -> Vec<(&K, &V)> {
        let mut out = vec![];
        if let Some(ref root) = self.root {
            root.traverse(&mut out);
        }
        out
    }
}

#[derive(Debug)]
struct BKTreeNode<K, V>
where
    K: Distance,
{
    key: K,
    value: V,
    children: HashMap<usize, BKTreeNode<K, V>>,
}

impl<K, V> BKTreeNode<K, V>
where
    K: Distance,
{
    /// Create a new BK Tree Node with the given (K, V) pair and empty HashMap of children
    fn new(key: K, value: V) -> Self {
        BKTreeNode {
            key,
            value,
            children: HashMap::new(),
        }
    }

    /// Insert a new (key, value) pair into this nodes children
    fn insert(&mut self, key: K, value: V) {
        // Get the distance between the current nodes key and the given key
        let distance = self.key.distance(&key);
        // If the child exists, traverse and insert from there.
        if let Some(child) = self.children.get_mut(&distance) {
            child.insert(key, value);
        } else {
            // otherwise, insert the current node into the children and with the given distance
            self.children.insert(distance, BKTreeNode::new(key, value));
        }
    }

    /// Find a key in the given childrens nodes
    fn find(&self, key: &K, leniency: usize) -> (Vec<&V>, Vec<&K>) {
        // Create a new tuple of empty vectors for exact and close matches
        let (mut exact, mut close) = (vec![], vec![]);
        // Get the distance between the current nodes key and then passed in key.
        let current_distance = self.key.distance(&key);
        // If the current distance is 0, it means its an exact match so push it to our "exact" matches
        if current_distance == 0 {
            exact.push(&self.value);
        // Otherwise, if the value is less than our leniency then add it to the close matches
        } else if current_distance <= leniency {
            close.push(&self.key);
        }

        // Saturing just means that the values will not overflow
        for i in
            current_distance.saturating_sub(leniency)..=current_distance.saturating_add(leniency)
        {
            // Because of how the tree works, we can traverse based off the leniency
            if let Some(child) = self.children.get(&i) {
                let mut result = child.find(key, leniency);
                exact.append(&mut result.0);
                close.append(&mut result.1);
            }
        }
        // return our vector of close and exact values
        return (exact, close);
    }

    fn traverse<'a>(&'a self, mut out: &mut Vec<(&'a K, &'a V)>) {
        if self.children.len() > 0 {
            self.children
                .iter()
                .for_each(|(_, child)| child.traverse(&mut out));
        } else {
            out.push((&self.key, &self.value));
        }
    }
}

/// This trait is used by the BKTree to determine the distance between 2 objects
/// when fuzzy searching. An example of this for strings is the Levenshtein distance,
/// Damerau-Levenshtein distance, Optimal string alignment distance, or a custom implementation.
pub trait Distance {
    /// Used to determine the "distance" between two objects.
    fn distance(&self, other: &Self) -> usize;
}

// We want to implement distance for String, and OSA is a good way to do so.
// This allows us to create a BKTree using Strings
impl Distance for String {
    fn distance(&self, other: &Self) -> usize {
        osa_distance(self, other)
    }
}

impl Distance for &str {
    fn distance(&self, other: &Self) -> usize {
        osa_distance(self, other)
    }
}

// Manual implementation of this function. Did not want to include a seperate library.
// https://docs.rs/strsim/0.9.2/src/strsim/lib.rs.html#263-307
pub fn osa_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();

    if a == b {
        return 0;
    } else if a_len == 0 {
        return b_len;
    } else if b_len == 0 {
        return a_len;
    }

    let mut prev_two_distances = Vec::with_capacity(b_len + 1);
    let mut prev_distances = Vec::with_capacity(b_len + 1);
    let mut current_distances = Vec::with_capacity(b_len + 1);

    let mut prev_a_char = char::MAX;
    let mut prev_b_char = char::MAX;

    for i in 0..(b_len + 1) {
        prev_two_distances.push(i);
        prev_distances.push(i);
        current_distances.push(0);
    }

    for (i, a_char) in a.chars().enumerate() {
        current_distances[0] = i + 1;

        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            current_distances[j + 1] = min(
                current_distances[j] + 1,
                min(prev_distances[j + 1] + 1, prev_distances[j] + cost),
            );

            if i > 0 && j > 0 && a_char != b_char && a_char == prev_b_char && b_char == prev_a_char
            {
                current_distances[j + 1] =
                    min(current_distances[j + 1], prev_two_distances[j - 1] + 1);
            }

            prev_b_char = b_char;
        }

        prev_two_distances.clone_from(&prev_distances);
        prev_distances.clone_from(&current_distances);
        prev_a_char = a_char;
    }

    current_distances[b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_DATA: [(&'static str, &'static str); 7] = [
        ("AMERICAN SCRAP PROCESSING INC", "SCRAP PROCESSING INC"), // 0
        ("ACP ACQUISITION CORP", "CP ACQUISITION CORP"),           // 1
        ("ADAMANT TECHNOLOGIES", "NT TECHNOLOGIES, INC."),         // 2
        ("zehn", "fünfzehn"),                                     // 3
        (
            // 4
            "Genesis - The Carpet Crawlers",
            "Genesys - The carpett craulers",
        ),
        (
            // 5
            "Genesis - (The Lamb Lies Down on Broadway) The Carpet Crawlers",
            "Genesys - (The Lamb Lies Down on Broadway) The Chamber of 32 Doors",
        ),
        (
            // 6
            "Genesis - (The Lamb Lies Down on Broadway) The Lamia",
            "Genesis - The Lamia",
        ),
    ];

    #[test]
    fn osa_1() {
        assert_eq!(0, osa_distance(TEST_DATA[0].0, TEST_DATA[0].0));
        assert_eq!(9, osa_distance(TEST_DATA[0].0, TEST_DATA[0].1));
        assert_eq!(1, osa_distance(TEST_DATA[1].0, TEST_DATA[1].1));
        assert_eq!(11, osa_distance(TEST_DATA[2].0, TEST_DATA[2].1));
        assert_eq!(4, osa_distance(TEST_DATA[3].0, TEST_DATA[3].1));
        assert_eq!(5, osa_distance(TEST_DATA[4].0, TEST_DATA[4].1));
        assert_eq!(14, osa_distance(TEST_DATA[5].0, TEST_DATA[5].1));
        assert_eq!(33, osa_distance(TEST_DATA[6].0, TEST_DATA[6].1));
    }
}
