use std::vec::Vec;

#[derive(Default, Debug, Clone)]
/// Can be used to implement your own custom Colony.
/// Most users should just use [Colony]
pub struct ColonyIndex {
    // ID -> Member Index
    id_to_index: Vec<usize>,
    // Member Index -> ID
    index_to_id: Vec<usize>,
    // Freed IDs which can be re-used.
    freed: Vec<usize>,
}

impl ColonyIndex {
    pub fn insert(&mut self, index: usize) -> usize {
        if let Some(id) = self.freed.pop() {
            self.id_to_index[id] = index;
            self.index_to_id[index as usize] = id;
            return id;
        }
        let id = self.id_to_index.len();
        self.id_to_index.push(index);
        self.index_to_id.push(id);
        id
    }

    pub fn to_index_unchecked(&self, id: usize) -> usize {
        self.id_to_index[id]
    }

    pub fn to_index(&self, id: usize) -> Option<usize> {
        let index = *self.id_to_index.get(id).unwrap_or(&std::usize::MAX);
        if index == std::usize::MAX {
            return None;
        }
        Some(index)
    }

    // Removal is always where they get you.
    // Always the most complicated part of any dynamic data structure.
    // 1: start
    // id_to_index: [2,0,1,3]
    //     elements: [A,B,C,D]
    // 2: remove id 2
    //                   v
    // id_to_index: [2,0,1,3]
    //     elements: [A,B,C,D]
    // 3:
    //                   v
    // id_to_index: [2,0,1,3]
    //                 v
    //     elements: [A,B,C,D]
    // 4: swap with last place
    // id_to_index: [2,0,1,3]
    //     elements: [A,D,C]
    // 5: update index
    // id_to_index: [2,0,1,1]
    //     elements: [A,D,C]
    pub fn remove(&mut self, target_id: usize, last_index: usize) -> Option<usize> {
        let target_index = *self.id_to_index.get(target_id).unwrap_or(&std::usize::MAX);
        if target_index == std::usize::MAX {
            return None;
        }
        let last_id = self.index_to_id[last_index];

        self.id_to_index[target_id] = std::usize::MAX;
        self.id_to_index[last_id] = target_index;
        self.index_to_id[target_index] = last_id;
        self.freed.push(target_id);
        Some(target_index)
    }
}

#[derive(Debug, Clone)]
/// # Colony
/// Cache-friendly packed associative data structure.
/// ```
/// # use packed_colony::Colony;
/// let mut library = Colony::new();
/// let book1 = library.insert("Foucault's Pendulum");
/// println!("{}", library[book1]);
/// ```
/// Suitable for real-time systems such as storing game entities,
/// drawables in rendering engines or similar. Provides ideal iteration
/// performance while allowing for associative lookup of specific elements.
/// Unlike the indexes of a `Vec`, the ids of a `Colony` remain stable as
/// elements are added or removed.
/// ### Advantages
/// * Very fast lookup (lookup is two array accesses)
/// * Underlying `Vec<T>` storage is accessible (as `elements`) and tightly-packed:
/// ```
/// # use packed_colony::Colony;
/// let mut scores = Colony::new();
/// for x in 1..100 {
///   scores.insert(x);
/// }
/// for score in scores.elements {
///   println!("{}", score);
/// }
/// ```
/// * Underlying `Vec` acts like a slab or pool allocator, amortising allocation cost
/// * Faster than a `HashMap` for lookup and Iteration
/// ### Disadvantages
/// * User does not pick the keys
/// * Keys may be re-used, meaning in:
/// ```
/// # use packed_colony::Colony;
/// let mut world = Colony::new();
/// let omega = world.insert("omega");
/// let star = world.insert("star");
/// world.remove(omega);
/// let gamma = world.insert("gamma");
/// ````
/// `omega` and `gamma` may be the same.
/// * elements are not pointer-stable
/// ## Implementation Notes
/// The Colony internally uses two lookup tables,
/// `id_to_index` and `index_to_id`.
/// A lookup is as simple as `elements[id_to_index[id]]`.
/// During removal, the removed element is swapped for the last
/// element in members, and the lookup tables are updated.
/// This naturally keeps all the data tightly packed.
pub struct Colony<T> {
    index: ColonyIndex,
    /// Manually adding or removing elements is not safe.
    pub elements: Vec<T>,
}

impl<T> Default for Colony<T> {
    fn default() -> Self {
        Self {
            index: ColonyIndex::default(),
            elements: Vec::new(),
        }
    }
}

impl<T> Colony<T> {
    pub fn new() -> Self {
        Colony::default()
    }

    pub fn insert(&mut self, entity: T) -> usize {
        let id = self.index.insert(self.elements.len());
        self.elements.push(entity);
        id
    }

    /// The Index trait is also supported.
    pub fn get(&self, id: usize) -> Option<&T> {
        if let Some(index) = self.index.to_index(id) {
            return self.elements.get(index);
        }
        None
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        if let Some(index) = self.index.to_index(id) {
            return self.elements.get_mut(index);
        }
        None
    }

    /// Idempotent, calling with invalid id will do nothing.
    pub fn remove(&mut self, id: usize) {
        if let Some(index) = self.index.remove(id, self.elements.len() - 1) {
            self.elements.swap_remove(index);
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn clear(&mut self) {
        self.index = ColonyIndex::default();
        self.elements.clear();
    }
}

impl<T> std::ops::Index<usize> for Colony<T> {
    type Output = T;

    fn index(&self, id: usize) -> &Self::Output {
        self.elements.index(self.index.to_index_unchecked(id))
    }
}

impl<T> std::ops::IndexMut<usize> for Colony<T> {
    fn index_mut(&mut self, id: usize) -> &mut Self::Output {
        self.elements.index_mut(self.index.to_index_unchecked(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut world = Colony::new();
        let id = world.insert(42);
        let value = world[id];
        world.remove(id);
        assert_eq!(value, 42);
        let id = world.insert(20);
        let value = world[id];
        assert_eq!(value, 20);
        let value2 = world[id];
        assert_eq!(value2, 20);
        world[id] = 89;
        let value3 = world[id];
        assert_eq!(value3, 89);
    }

    #[test]
    fn safe_api() {
        let mut world = Colony::default();
        let a = world.insert("A");
        let b = world.insert("B");
        let c = world.insert("C");
        assert_eq!(world[a], "A");
        assert_eq!(world[b], "B");
        assert_eq!(world[c], "C");
        assert_eq!(world.len(), 3);
        assert_eq!(world.get(1337), None);
        world.remove(1337);
        assert_eq!(*world.get(a).unwrap(), "A");
        world.remove(a);
        assert_eq!(world.get(a), None);
        world.remove(a);
        world.clear();
        world.clear();
    }
}
