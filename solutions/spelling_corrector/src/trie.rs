use std::ops::Index;

/// A `TrieMap<T>` maps sequences of natural numbers to `T`s. The trie
/// has a branching factor, `factor`, that determines the range of
/// natural numbers accepted: [0, factor).
#[derive(Debug, PartialEq, Eq)]
pub struct TrieMap<T> {
    /// The root node of the trie:
    node:   Node<T>,
    /// The branching factor (size of the alphabet):
    factor: usize,
    /// The value `None`, which allows us to return a `None` borrowed
    /// from the map when necessary.
    none:   Option<T>,
}

/// A trie node may have a value, and has a vector of nullable pointers to
/// child nodes.
#[derive(Debug, PartialEq, Eq)]
struct Node<T> {
    value:    Option<T>,
    children: Box<[Option<Box<Node<T>>>]>,
}

/// A cursor marks a position in the trie and can be used to traverse it.
/// In particular, we can find out the value at the current node, and we
/// can descend to a child node based on the child index.
#[derive(Copy, Clone, Debug)]
pub struct Cursor<'a, T: 'a> {
    node: &'a Node<T>,
}

/// A mutable cursor marks a position in the trie, and can be used to
/// traverse and modify it the trie.
#[derive(Debug)]
pub struct CursorMut<'a, T: 'a> {
    node:   &'a mut Node<T>,
    factor: usize,
}

impl<T> Node<T> {
    /// Creates a new node with the given branching factor.
    fn new(factor: usize) -> Self {
        let mut children = Vec::with_capacity(factor);

        for _ in 0 .. factor {
            children.push(None);
        }

        Node {
            value:    None,
            children: children.into_boxed_slice(),
        }
    }

    /// Gets a cursor pointing to this node.
    fn cursor(&self) -> Cursor<T> {
        Cursor { node: self }
    }

    /// Gets a mutable cursor pointing to this mutable node, with the given
    /// branching factor. (The branching factor is needed in order to
    /// create new nodes.)
    fn cursor_mut(&mut self, factor: usize) -> CursorMut<T> {
        CursorMut {
            node: &mut *self,
            factor: factor,
        }
    }

    /// Looks up a key in the trie, returning its value or the given default.
    fn lookup<'a, 'b, I>(&'a self, key: I, default: &'a Option<T>)
        -> &'a Option<T>
    where I: IntoIterator<Item=&'b usize>
    {
        let mut cursor = self.cursor();

        for &each in key {
            if !cursor.descend(each) { return default }
        }

        cursor.value()
    }
}

impl<'a, T> Cursor<'a, T> {
    /// Gets the value of the node this cursor points to.
    pub fn value(&self) -> &'a Option<T> {
        &self.node.value
    }

    /// Gets a cursor to the `key`th child.
    pub fn child(&self, key: usize) -> Option<Self> {
        self.node.children[key].as_ref().map(|n| n.cursor())
    }

    /// Updates this cursor to point to the `key`th child.
    /// Returns whether the cursor actually moved; the result will be
    /// `false` if the child doesn't exist.
    pub fn descend(&mut self, key: usize) -> bool {
        match self.child(key) {
            None         => false,
            Some(cursor) => { *self = cursor; true },
        }
    }
}

impl<'a, T> CursorMut<'a, T> {
    /// Borrows an immutable cursor from this mutable cursor.
    pub fn freeze(&self) -> Cursor<T> {
        self.node.cursor()
    }

    /// Gets the value of the node this cursor points to.
    pub fn value(&mut self) -> &mut Option<T> {
        &mut self.node.value
    }

    /// Borrows a cursor to the `key`th child.
    pub fn child(&'a mut self, key: usize) -> Option<Self> {
        let factor = self.factor;
        self.node.children[key].as_mut().map(|n| n.cursor_mut(factor))
    }

    /// Consumes a cursor, returning a cursor to the `key`th child.
    pub fn into_child(self, key: usize) -> Option<Self> {
        let factor = self.factor;
        self.node.children[key].as_mut().map(|n| n.cursor_mut(factor))
    }

    /// Adds a child at the given key position, if there isn't one
    /// already, and returns a `CursorMut` pointing to the child.
    pub fn child_add(&'a mut self, key: usize) -> Self {
        match &mut self.node.children[key] {
            &mut Some(ref mut child) => child,
            otherwise => {
                *otherwise = Some(Box::new(Node::new(self.factor)));
                otherwise.as_mut().unwrap()
            },
        }
        .cursor_mut(self.factor)
    }

    /// Consumes the cursor and adds a child at the given key position
    /// if there isn't one already, returning a `CursorMut` pointing
    /// to the child.
    pub fn into_child_add(self, key: usize) -> Self {
        match &mut self.node.children[key] {
            &mut Some(ref mut child) => child,
            otherwise => {
                *otherwise = Some(Box::new(Node::new(self.factor)));
                otherwise.as_mut().unwrap()
            },
        }
        .cursor_mut(self.factor)
    }
}

impl<T> TrieMap<T> {
    /// Creates a new, empty `TrieMap` with the given branching factor.
    pub fn new(factor: usize) -> Self {
        TrieMap {
            node:   Node::new(factor),
            factor: factor,
            none:   None,
        }
    }

    /// Returns a cursor pointing to the root of the trie.
    pub fn cursor(&self) -> Cursor<T> {
        self.node.cursor()
    }

    /// Returns a mutable cursor pointing to the root of the trie.
    pub fn cursor_mut(&mut self) -> CursorMut<T> {
        self.node.cursor_mut(self.factor)
    }

    /// Checks whether the trie contains a particular key.
    pub fn contains(&self, key: &[usize]) -> bool {
        self[key].is_some()
    }
}

impl<'a, T> Index<&'a [usize]> for TrieMap<T> {
    type Output = Option<T>;

    fn index(&self, key: &[usize]) -> &Option<T> {
        self.node.lookup(key, &self.none)
    }
}

