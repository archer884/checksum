pub trait CompareAgainstHead {
    fn all_items_match(self) -> bool;
}

impl<T: IntoIterator> CompareAgainstHead for T
where
    <T as IntoIterator>::Item: Eq,
{
    fn all_items_match(self) -> bool {
        let mut items = self.into_iter();

        let head = match items.next() {
            Some(head) => head,
            None => return true,
        };

        items.all(|x| x == head)
    }
}
