use std::cmp::Ordering;
use std::collections::HashMap;

use slab::Slab;

use crate::heap::Heap;

mod heap;

struct TreeNode<T> {
    element: T,
    marked: bool,
    children: Slab<TreeNode<T>>,
}

impl<T> TreeNode<T> {
    fn new(element: T) -> TreeNode<T> {
        TreeNode { element, marked: false, children: Slab::new() }
    }

    fn degree(&self) -> usize {
        self.children.len()
    }
}

struct FibonacciHeap<T> {
    trees: Slab<TreeNode<T>>,
    min_element: usize,
    count: usize,
}

impl<T: Ord> heap::Heap<T> for FibonacciHeap<T> {
    fn find_minimum(&self) -> Option<&T> {
        match self.trees.get(self.min_element) {
            None => None,
            Some(elem) => Some(&elem.element)
        }
    }

    fn merge(mut self, mut heap_to_merge: Self) -> Self {
        for tree in heap_to_merge.trees.drain() {
            self.insert_tree(tree)
        }
        self.count += heap_to_merge.count;
        self
    }

    fn insert(&mut self, element: T) {
        let new_minimum = self.is_new_minimum(&element);
        let position = self.trees.insert(TreeNode::new(element));
        if new_minimum {
            self.min_element = position;
        }
        self.count += 1;
    }


    fn extract_minimum(&mut self) -> Option<T> {
        // No trees in the heap, return None.
        if !self.trees.contains(self.min_element) {
            return None;
        }

        // The following comments were extracted from the Fibonacci heap article on Wikipedia.
        // https://en.wikipedia.org/w/index.php?title=Fibonacci_heap&oldid=944266509

        // Operation extract minimum (same as delete minimum) operates in three phases.
        // First we take the root containing the minimum element and remove it.
        let mut removed_root = self.trees.remove(self.min_element);
        self.count -= 1;

        // Its children will become roots of new trees.
        // If the number of children was d, it takes time O(d) to process all new roots and the
        // potential increases by d−1. Therefore, the amortized running time of this phase is
        // O(d) = O(log n).
        for child in removed_root.children.drain() {
            self.trees.insert(child);
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
        let mut degrees: HashMap<usize, usize> = HashMap::new();
        let indices: Vec<usize> = self.trees.iter().map(|(index, _)| index).collect();
        for index in indices {
            self.insertAndMergeDegree(&mut degrees, index);
        }


        // In the third phase we check each of the remaining roots and find the minimum. This
        // takes O(log n) time and the potential does not change. The overall amortized running
        // time of extract minimum is therefore O(log n).
        let (index, _) = self.trees.iter().min_by_key(|(index, tree)| tree.degree()).unwrap();
        self.min_element = index;

        Some(removed_root.element)
    }

    fn decrease_key(&mut self, old_element: T, new_element: T) {
        unimplemented!()
    }

    fn delete(&mut self, element: T) {
        self.count -= 1;
        unimplemented!()
    }
}


impl<T: Ord> FibonacciHeap<T> {
    fn new() -> FibonacciHeap<T> {
        FibonacciHeap {
            trees: Slab::new(),
            min_element: 0,
            count: 0,
        }
    }

    fn is_new_minimum(&self, element: &T) -> bool {
        if let Some(current_minimum) = self.find_minimum() {
            return element.cmp(current_minimum) == Ordering::Less;
        }
        false
    }

    fn insert_tree(&mut self, tree: TreeNode<T>) {
        let new_minimum = self.is_new_minimum(&tree.element);
        let position = self.trees.insert(tree);
        if new_minimum {
            self.min_element = position;
        }
    }

    fn insertAndMergeDegree(&mut self, degreesMap: &mut HashMap<usize, usize>, tree_to_insert_index: usize) {
        let degree = self.trees.get(tree_to_insert_index).unwrap().degree();
        if !degreesMap.contains_key(&degree) {
            degreesMap.insert(degree, tree_to_insert_index);
            return;
        }

        let tree_to_merge_index = degreesMap.remove(&degree).unwrap();

        let mut tree_to_merge1 = self.trees.remove(tree_to_merge_index);
        let mut tree_to_merge2 = self.trees.remove(tree_to_insert_index);

        if tree_to_merge1.element.cmp(&tree_to_merge2.element) == Ordering::Less {
            tree_to_merge1.children.insert(tree_to_merge2);
            let new_index = self.trees.insert(tree_to_merge1);
            self.insertAndMergeDegree(degreesMap, new_index);
        } else {
            tree_to_merge2.children.insert(tree_to_merge1);
            let new_index = self.trees.insert(tree_to_merge2);
            self.insertAndMergeDegree(degreesMap, new_index);
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::FibonacciHeap;
    use crate::heap::Heap;

    #[test]
    fn it_works() {
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
    }
}

