use std::cmp::Ordering;
use std::collections::HashMap;

use slab::Slab;

use crate::heap::Heap;

mod heap;

struct NodeID(usize, usize);

struct TreeNode<T> {
    parent: Option<usize>,
    element: T,
    marked: bool,
    children: Vec<usize>,
    handle_id: usize,
}

impl<T> TreeNode<T> {
    fn new(element: T, parent: Option<usize>, handle_id: usize) -> TreeNode<T> {
        TreeNode {
            parent,
            element,
            marked: false,
            children: Vec::new(),
            handle_id,
        }
    }

    fn degree(&self) -> usize {
        self.children.len()
    }
}


struct FibonacciHeap<T> {
    nodes: Slab<TreeNode<T>>,
    trees: Vec<usize>,
    min_element: usize,
    id_counter: usize,
}

impl<T: Ord> heap::Heap<T> for FibonacciHeap<T> {
    type Handle = NodeID;

    fn find_minimum(&self) -> Option<&T> {
        match self.nodes.get(self.min_element) {
            None => None,
            Some(elem) => Some(&elem.element),
        }
    }

    fn merge(mut self, mut heap_to_merge: Self) -> Self {
        for tree_id in heap_to_merge.trees.iter() {
            let tree = heap_to_merge.nodes.remove(*tree_id);
            self.insert_tree(tree);
        }
        self
    }

    fn insert(&mut self, element: T) -> Self::Handle {
        self.id_counter += 1;
        let tree = TreeNode::new(element, None, self.id_counter);
        NodeID(self.insert_tree(tree), self.id_counter)
    }

    fn extract_minimum(&mut self) -> Option<T> {
        // No trees in the heap, return None.
        if self.trees.is_empty() {
            return None;
        }

        // The following comments were extracted from the Fibonacci heap article on Wikipedia.
        // https://en.wikipedia.org/w/index.php?title=Fibonacci_heap&oldid=944266509

        // Operation extract minimum (same as delete minimum) operates in three phases.
        // First we take the root containing the minimum element and remove it.
        let removed_root = self.remove_tree(self.min_element);

        // Its children will become roots of new trees.
        // If the number of children was d, it takes time O(d) to process all new roots and the
        // potential increases by d−1. Therefore, the amortized running time of this phase is
        // O(d) = O(log n).
        for child in removed_root.children.iter() {
            self.trees.push(*child);
        }

        if self.trees.is_empty() {
            return Some(removed_root.element);
        }

        // However to complete the extract minimum operation, we need to update the pointer to the
        // root with minimum key. Unfortunately there may be up to n roots we need to check.
        // In the second phase we therefore decrease the number of roots by successively linking
        // together roots of the same degree. When two roots u and v have the same degree, we make
        // one of them a child of the other so that the one with the smaller key remains the root.
        // Its degree will increase by one. This is repeated until every root has a different
        // degree.
        // To find trees of the same degree efficiently we use an array of length O(log n)
        // in which we keep a pointer to one root of each degree. When a second root is found of the
        // same degree, the two are linked and the array is updated. The actual running time is
        // O(log n + m) where m is the number of roots at the beginning of the second phase.
        // At the end we will have at most O(log n) roots (because each has a different degree).
        // Therefore, the difference in the potential function from before this phase to after it
        // is: O(log n) − m, and the amortized running time is then at most
        // O(log n + m) + c(O(log n) − m).
        // With a sufficiently large choice of c, this simplifies to O(log n).
        let mut degrees_map: HashMap<usize, usize> = HashMap::new();
        for tree_to_insert_id in self.trees.clone().iter() {
            self.merge_or_merge_same_degrees_tree(*tree_to_insert_id, &mut degrees_map)
        }
        self.trees = degrees_map.values().map(|v| *v).collect();

        // In the third phase we check each of the remaining roots and find the minimum. This
        // takes O(log n) time and the potential does not change. The overall amortized running
        // time of extract minimum is therefore O(log n).
        self.min_element = *self
            .trees
            .iter()
            .map(|tree_id| (tree_id, self.nodes.get(*tree_id).unwrap()))
            .min_by_key(|(_, tree)| tree.degree())
            .map(|(tree_id, _)| tree_id)
            .unwrap();

        Some(removed_root.element)
    }

    fn decrease_key(&mut self, handle: &Self::Handle, new_element: T) {
        // The following comments were extracted from the Fibonacci heap article on Wikipedia.
        // https://en.wikipedia.org/w/index.php?title=Fibonacci_heap&oldid=944266509

        let node_id = handle.0;
        let node = self.nodes.get(node_id);
        if let None = node {
            return; // Not in the heap.
        }
        let node = node.unwrap();
        if node.handle_id != handle.1 {
            return; // Handle refers to a node that was deleted.
        }
        if new_element.cmp(&node.element) == Ordering::Greater {
            return; // New element is greater than existing one.
        }

        if let None = node.parent {
            // Root node, just update the value and the minimum.
            self.nodes.get_mut(node_id).unwrap().element = new_element;
            if self.is_minimum(self.nodes.get(node_id).unwrap()) {
                self.min_element = node_id;
            }
            return;
        }

        let parent_id = node.parent.unwrap();
        let parent = self.nodes.get(parent_id).unwrap();
        if parent.element.cmp(&new_element) == Ordering::Less {
            // Heap property not violated, nothing to do.
            return;
        }

        // Operation decrease key will take the node, decrease the key and if the heap property
        // becomes violated (the new key is smaller than the key of the parent), the node is cut
        // from its parent. If the parent is not a root, it is marked.
        // If it has been marked already, it is cut as well and its parent is marked.
        // We continue upwards until we reach either the root or an unmarked node.
        self.mark_or_cut(node_id);

        // Now we set the minimum pointer to the decreased value if it is the new minimum. In the
        // process we create some number, say k, of new trees. Each of these new trees except
        // possibly the first one was marked originally but as a root it will become unmarked. One
        // node can become marked. Therefore, the number of marked nodes changes by
        // −(k − 1) + 1 = − k + 2. Combining these 2 changes, the potential changes by
        // 2(−k + 2) + k = −k + 4. The actual time to perform the cutting was O(k), therefore
        // (again with a sufficiently large choice of c) the amortized running time is constant.

        self.nodes.get_mut(node_id).unwrap().element = new_element;
        if self.is_minimum(self.nodes.get(node_id).unwrap()) {
            self.min_element = node_id
        }
    }

    fn delete(&mut self, element: T) {
        unimplemented!()
    }
}

impl<T: Ord> FibonacciHeap<T> {
    fn new() -> FibonacciHeap<T> {
        FibonacciHeap {
            nodes: Slab::new(),
            trees: Vec::new(),
            min_element: 0,
            id_counter: 0,
        }
    }

    fn is_minimum(&self, tree: &TreeNode<T>) -> bool {
        if let Some(min) = self.find_minimum() {
            return tree.element.cmp(min) == Ordering::Less;
        }
        false
    }

    fn remove_tree(&mut self, tree_id_to_remove: usize) -> TreeNode<T> {
        let index_to_remove = self
            .trees
            .iter()
            .enumerate()
            .filter(|(_, tree_id)| tree_id_to_remove == **tree_id)
            .map(|(i, _)| i)
            .next()
            .unwrap();

        self.trees.swap_remove(index_to_remove);
        return self.nodes.remove(tree_id_to_remove);
    }

    fn insert_tree(&mut self, tree: TreeNode<T>) -> usize {
        let is_minimum = self.is_minimum(&tree);
        let tree_id = self.nodes.insert(tree);
        self.trees.push(tree_id);
        if is_minimum {
            self.min_element = tree_id;
        }
        tree_id
    }

    fn merge_or_merge_same_degrees_tree(
        &mut self,
        tree_to_insert_id: usize,
        degrees_map: &mut HashMap<usize, usize>,
    ) {
        let tree_to_insert = self.nodes.get(tree_to_insert_id).unwrap();
        let degree = tree_to_insert.degree();

        if !degrees_map.contains_key(&degree) {
            degrees_map.insert(degree, tree_to_insert_id);
            return;
        }

        let tree_to_merge_id = degrees_map.remove(&degree).unwrap();
        let tree_to_merge = self.nodes.get(tree_to_merge_id).unwrap();

        if tree_to_merge.element.cmp(&tree_to_insert.element) == Ordering::Less {
            self.nodes
                .get_mut(tree_to_merge_id)
                .unwrap()
                .children
                .push(tree_to_insert_id);
            self.nodes.get_mut(tree_to_insert_id).unwrap().parent = Some(tree_to_merge_id);
            self.merge_or_merge_same_degrees_tree(tree_to_merge_id, degrees_map);
        } else {
            self.nodes
                .get_mut(tree_to_insert_id)
                .unwrap()
                .children
                .push(tree_to_merge_id);
            self.nodes.get_mut(tree_to_merge_id).unwrap().parent = Some(tree_to_insert_id);
            self.merge_or_merge_same_degrees_tree(tree_to_insert_id, degrees_map);
        }
    }

    fn cut(&mut self, node_id: usize, parent_id: usize) {
        let mut parent = self.nodes.get_mut(parent_id).unwrap();
        parent.children = parent
            .children
            .iter()
            .filter(|child| **child != node_id)
            .map(|child| *child)
            .collect();

        let mut node = self.nodes.get_mut(node_id).unwrap();
        node.parent = None;

        self.trees.push(node_id);
    }
    fn mark_or_cut(&mut self, node_id: usize) {
        let node = self.nodes.get(node_id).unwrap();
        let parent_id = node.parent;
        if let None = parent_id {
            // Root node.
            return;
        }
        let parent_id = parent_id.unwrap();
        let mut parent = self.nodes.get_mut(parent_id).unwrap();
        if parent.marked {
            // Already marked, we need to cut it.
            self.cut(node_id, parent_id);
            self.mark_or_cut(parent_id);
        } else {
            // Not marked yet.
            parent.marked = true;
            self.cut(node_id, parent_id)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FibonacciHeap;
    use crate::heap::Heap;

    #[test]
    fn fib_heap_test() {
        let mut a: FibonacciHeap<i32> = FibonacciHeap::new();
        assert_eq!(a.find_minimum(), None);
        assert_eq!(a.extract_minimum(), None);

        a.insert(42);
        assert_eq!(a.find_minimum(), Some(&42));
        a.insert(10);
        assert_eq!(a.find_minimum(), Some(&10));

        let mut b: FibonacciHeap<i32> = FibonacciHeap::new();
        b.insert(2);

        let mut a = a.merge(b);
        assert_eq!(a.find_minimum(), Some(&2));

        assert_eq!(a.extract_minimum(), Some(2));

        assert_eq!(a.find_minimum(), Some(&10));
        assert_eq!(a.extract_minimum(), Some(10));
        assert_eq!(a.extract_minimum(), Some(42));
        assert_eq!(a.extract_minimum(), None);
        assert_eq!(a.extract_minimum(), None);

        let handle42 = a.insert(42);
        let handle10 = a.insert(10);
        assert_eq!(a.find_minimum(), Some(&10));

        a.decrease_key(&handle42, 2);
        assert_eq!(a.find_minimum(), Some(&2));

        a.decrease_key(&handle10, 1);
        assert_eq!(a.find_minimum(), Some(&1));
    }
}
