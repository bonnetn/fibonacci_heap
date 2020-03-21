pub trait Heap<T: Ord> {
    fn find_minimum(&self) -> Option<&T>;
    fn merge(self, heap_to_merge: Self) -> Self;
    fn insert(&mut self, element: T);
    fn extract_minimum(&mut self) -> Option<T>;
    fn decrease_key(&mut self, old_element: T, new_element: T);
    fn delete(&mut self, element: T);
}
