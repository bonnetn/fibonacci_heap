pub trait Heap<T: Ord> {
    type Handle;

    fn find_minimum(&self) -> Option<&T>;
    fn merge(self, heap_to_merge: Self) -> Self;
    fn insert(&mut self, element: T) -> Self::Handle;
    fn extract_minimum(&mut self) -> Option<T>;
    fn decrease_key(&mut self, handle: &Self::Handle, new_element: T);
    fn delete(&mut self, element: T);
}
