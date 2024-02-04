use std::{collections::HashMap, result};

#[derive(Debug)]
pub struct QuadTreeInsertError;

#[derive(Debug, Default, Clone)]
pub struct QuadTree<T> {
    pub nodes: HashMap<u64, T>,
    handle_counter: u64,

    min_size: f32,

    tree: QuadTreeInner,
}

impl<T> QuadTree<T> {
    pub fn new(max_nodes: usize, top_left: (f32, f32), bot_right: (f32, f32)) -> Self {
        Self {
            nodes: HashMap::new(),
            handle_counter: 0,
            min_size: 1.0,
            tree: QuadTreeInner::new(max_nodes, top_left, bot_right),
        }
    }

    pub fn insert(&mut self, item: T, pos: (f32, f32)) -> Result<(), QuadTreeInsertError> {
        if self.tree.insert(pos, self.handle_counter).is_ok() {
            self.nodes.insert(self.handle_counter, item);
            self.handle_counter += 1;

            return Ok(());
        }

        Err(QuadTreeInsertError)
    }

    pub fn lines(&self) -> Vec<((f32, f32), (f32, f32))> {
        self.tree.lines()
    }

    pub fn search_radius(&self, pos: (f32, f32), r: f32) -> Vec<&T> {
        let mut result = Vec::new();
        for k in self.tree.search_radius(pos, r) {
            result.push(self.nodes.get(&k).unwrap());
        }

        result
    }

    pub fn search_radius_ids(&self, pos: (f32, f32), r: f32) -> Vec<u64> {
        self.tree.search_radius(pos, r)
    }

    pub fn remove(&mut self, id: u64, pos: (f32, f32)) -> Option<T> {
        self.tree.remove(id, pos);
        self.nodes.remove(&id)
    }
}

#[derive(Debug, Default, Clone)]
struct QuadTreeInner {
    nodes: Vec<u64>,
    max_nodes: usize,

    top_left: (f32, f32),
    bot_right: (f32, f32),

    quads: Option<Box<[Self; 4]>>,
}

impl QuadTreeInner {
    fn new(max_nodes: usize, top_left: (f32, f32), bot_right: (f32, f32)) -> Self {
        Self {
            nodes: Vec::new(),
            max_nodes,
            top_left,
            bot_right,
            quads: None,
        }
    }

    fn insert(&mut self, pos: (f32, f32), handle: u64) -> Result<(), QuadTreeInsertError> {
        todo!()
    }

    fn in_boundary(&self, pos: (f32, f32)) {
        todo!()
    }

    fn split(&mut self) {
        todo!()
    }

    fn search_radius(&self, pos: (f32, f32), r: f32) -> Vec<u64> {
        todo!()
    }

    fn contains_circle(&self, pos: (f32, f32), r: f32) -> bool {
        todo!()
    }

    fn remove(&mut self, handle: u64, pos: (f32, f32)) -> Option<u64> {
        todo!()
    }

    fn lines(&self) -> Vec<((f32, f32), (f32, f32))> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
