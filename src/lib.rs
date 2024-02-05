use std::collections::HashMap;

pub trait Position {
    fn position(&self) -> (f32, f32);
}

#[derive(Debug)]
pub struct QuadTreeInsertError;

#[derive(Debug, Default, Clone)]
pub struct QuadTree<T>
where
    T: Position,
{
    pub nodes: HashMap<u64, T>,
    handle_counter: u64,

    tree: QuadTreeInner,
}

impl<T> QuadTree<T>
where
    T: Position,
{
    pub fn new(
        max_nodes: usize,
        min_size: f32,
        top_left: (f32, f32),
        bot_right: (f32, f32),
    ) -> Self {
        Self {
            nodes: HashMap::new(),
            handle_counter: 0,
            tree: QuadTreeInner::new(max_nodes, min_size, top_left, bot_right),
        }
    }

    pub fn insert(&mut self, item: T, pos: (f32, f32)) -> Result<(), QuadTreeInsertError> {
        let mut splits = self.tree.insert(pos, self.handle_counter)?;

        self.nodes.insert(self.handle_counter, item);
        self.handle_counter += 1;

        while let Some(id) = splits.pop() {
            let mut new_splits = self
                .tree
                .insert(self.nodes.get(&id).unwrap().position(), id)?;
            splits.append(&mut new_splits);
        }

        Ok(())
    }

    pub fn lines(&self) -> Vec<((f32, f32), (f32, f32))> {
        self.tree.lines()
    }

    pub fn search_radius(&self, pos: (f32, f32), r: f32) -> Vec<&T> {
        let mut result = Vec::new();
        for k in self.tree.search_radius(pos, r) {
            let position = self.nodes.get(&k).unwrap().position();
            // Yes I realize that abs() isn't nessassary, but the compiler *should* optimize them out, and it's more clear.
            // NOTE: You can probably optimize here; if the quad is contained in the radius, you don't need this check.
            //   Also, a version of this that loosens the radius check a bit by not checking min size areas would be nice.
            if (position.0 - pos.0).abs().powi(2) + (position.1 - pos.1).abs().powi(2) <= r * r {
                result.push(self.nodes.get(&k).unwrap());
            }
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

    pub fn reinsert(&mut self, id: u64, old_pos: (f32, f32)) -> Result<(), QuadTreeInsertError> {
        self.tree.remove(id, old_pos);

        let mut splits = self
            .tree
            .insert(self.nodes.get(&id).unwrap().position(), id)?;
        while let Some(id) = splits.pop() {
            let mut new_splits = self
                .tree
                .insert(self.nodes.get(&id).unwrap().position(), id)?;
            splits.append(&mut new_splits);
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
struct QuadTreeInner {
    nodes: Vec<u64>,
    max_nodes: usize,

    min_size: f32,

    top_left: (f32, f32),
    bot_right: (f32, f32),

    quads: Option<Box<[Self; 4]>>,
}

impl QuadTreeInner {
    fn new(max_nodes: usize, min_size: f32, top_left: (f32, f32), bot_right: (f32, f32)) -> Self {
        Self {
            nodes: Vec::new(),
            max_nodes,
            min_size,
            top_left,
            bot_right,
            quads: None,
        }
    }

    fn insert(&mut self, pos: (f32, f32), handle: u64) -> Result<Vec<u64>, QuadTreeInsertError> {
        if !self.in_boundary(pos) {
            return Err(QuadTreeInsertError);
        }

        if self.quads.is_none() {
            if self.nodes.len() < self.max_nodes
                || (self.top_left.0 - self.bot_right.0).abs() <= self.min_size
                || (self.top_left.1 - self.bot_right.1).abs() <= self.min_size
            {
                // Forces nodes to always be sorted
                let idx = self.nodes.partition_point(|&x| x < handle);
                self.nodes.insert(idx, handle);

                return Ok(Vec::new());
            } else {
                return Ok(self.split());
            }
        }

        // quads is now always Some
        if (self.top_left.0 + self.bot_right.0) / 2.0 >= pos.0 {
            if (self.top_left.1 + self.bot_right.1) / 2.0 >= pos.1 {
                // Insert top left
                self.quads.as_mut().unwrap()[0].insert(pos, handle)?;
            } else {
                // insert bot left
                self.quads.as_mut().unwrap()[1].insert(pos, handle)?;
            }
        } else if (self.top_left.1 + self.bot_right.1) / 2.0 >= pos.1 {
            // Insert top right
            self.quads.as_mut().unwrap()[2].insert(pos, handle)?;
        } else {
            // Insert bot right
            self.quads.as_mut().unwrap()[3].insert(pos, handle)?;
        }

        Ok(Vec::new())
    }

    fn in_boundary(&self, pos: (f32, f32)) -> bool {
        pos.0 >= self.top_left.0
            && pos.0 <= self.bot_right.0
            && pos.1 >= self.top_left.1
            && pos.1 <= self.bot_right.1
    }

    fn split(&mut self) -> Vec<u64> {
        let top_left_quad = Self::new(
            self.max_nodes,
            self.min_size,
            self.top_left,
            (
                (self.top_left.0 + self.bot_right.0) / 2.0,
                (self.top_left.1 + self.bot_right.1) / 2.0,
            ),
        );

        let top_right_quad = Self::new(
            self.max_nodes,
            self.min_size,
            ((self.top_left.0 + self.bot_right.0) / 2.0, self.top_left.1),
            (
                (self.top_left.0 + self.bot_right.0) / 2.0,
                (self.top_left.1 + self.bot_right.1) / 2.0,
            ),
        );

        let bot_left_quad = Self::new(
            self.max_nodes,
            self.min_size,
            (self.top_left.0, (self.top_left.1 + self.bot_right.1) / 2.0),
            ((self.top_left.0 + self.bot_right.0) / 2.0, self.bot_right.1),
        );

        let bot_right_quad = Self::new(
            self.max_nodes,
            self.min_size,
            (
                (self.top_left.0 + self.bot_right.0) / 2.0,
                (self.top_left.1 + self.bot_right.1) / 2.0,
            ),
            self.bot_right,
        );

        self.quads = Some(Box::new([
            top_left_quad,
            top_right_quad,
            bot_left_quad,
            bot_right_quad,
        ]));

        self.nodes.drain(..).collect()
    }

    fn search_radius(&self, pos: (f32, f32), r: f32) -> Vec<u64> {
        match &self.quads {
            Some(quads) => {
                let mut nodes = Vec::new();
                quads.iter().for_each(|quad| {
                    if quad.contains_circle(pos, r) {
                        nodes.extend(quad.search_radius(pos, r))
                    }
                });
                nodes
            }
            None => self.nodes.clone(),
        }
    }

    fn contains_circle(&self, pos: (f32, f32), r: f32) -> bool {
        let mut test = pos;

        if pos.0 < self.top_left.0 {
            test.0 = self.top_left.0;
        } else if pos.0 > self.bot_right.0 {
            test.0 = self.bot_right.0;
        }
        if pos.1 < self.top_left.1 {
            test.1 = self.top_left.1;
        } else if pos.1 > self.bot_right.1 {
            test.1 = self.bot_right.1;
        }

        (test.0 - pos.0).abs().powi(2) + (test.1 - pos.1).abs().powi(2) <= r.powi(2)
    }

    fn remove(&mut self, handle: u64, pos: (f32, f32)) -> Option<u64> {
        if !self.in_boundary(pos) {
            return None;
        }

        if self.quads.is_none() {
            // The nodes should always be sorted, because the ids
            match self.nodes.binary_search(&handle) {
                Ok(n) => return Some(self.nodes.remove(n)),
                Err(_) => return None,
            }
        }

        // quads is now always Some
        if (self.top_left.0 + self.bot_right.0) / 2.0 >= pos.0 {
            if (self.top_left.1 + self.bot_right.1) / 2.0 >= pos.1 {
                // Insert top left
                self.quads.as_mut().unwrap()[0].remove(handle, pos)
            } else {
                // insert bot left
                self.quads.as_mut().unwrap()[1].remove(handle, pos)
            }
        } else if (self.top_left.1 + self.bot_right.1) / 2.0 >= pos.1 {
            // Insert top right
            self.quads.as_mut().unwrap()[2].remove(handle, pos)
        } else {
            // Insert bot right
            self.quads.as_mut().unwrap()[3].remove(handle, pos)
        }

        // TODO: There should be something here about removing a split when the number of child nodes is (zero? since this
        // is for moving objects, unsplitting when the number of child nodes equals the max nodes for a cell may result in constantly
        // splitting and unsplitting)
    }

    fn lines(&self) -> Vec<((f32, f32), (f32, f32))> {
        let mut lines = vec![
            (self.top_left, (self.top_left.0, self.bot_right.1)),
            (self.top_left, (self.bot_right.0, self.top_left.1)),
            (self.bot_right, (self.top_left.0, self.bot_right.1)),
            (self.bot_right, (self.bot_right.0, self.top_left.1)),
        ];

        match &self.quads {
            Some(quads) => {
                for quad in quads.iter() {
                    lines.extend(quad.lines());
                }
            }
            None => {}
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Default, Clone, Copy)]
    struct TestStruct {
        pos: (f32, f32),
    }

    impl Position for TestStruct {
        fn position(&self) -> (f32, f32) {
            self.pos
        }
    }

    #[test]
    fn test_new_quadtree() {
        dbg!(QuadTree::<TestStruct>::new(10, 1.0, (0.0, 0.0), (1.0, 1.0)));
    }

    #[test]
    fn test_split_quadtree() {
        let mut qt = QuadTree::<TestStruct>::new(5, 0.01, (0.0, 0.0), (1.0, 1.0));

        let n = 10_000;
        for i in 0..n {
            let t = TestStruct {
                pos: (i as f32 / n as f32, i as f32 / n as f32),
            };
            let pos = t.position();
            qt.insert(t, pos)
                .unwrap_or_else(|_| panic!("Error when inserting item! {:?}", t));
        }
    }

    #[test]
    fn test_stacked_units() {
        let mut qt = QuadTree::<TestStruct>::new(5, 0.01, (0.0, 0.0), (1.0, 1.0));

        let n = 10_000;
        for _ in 0..n {
            let t = TestStruct { pos: (0.1, 0.1) };
            let pos = t.position();
            qt.insert(t, pos)
                .unwrap_or_else(|_| panic!("Error when inserting item! {:?}", t));
        }
    }
}
