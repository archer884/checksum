pub trait IsUniform {
    fn is_uniform(self) -> bool;
}

impl<T: IntoIterator> IsUniform for T
where
    T::Item: Eq,
{
    fn is_uniform(self) -> bool {
        let mut items = self.into_iter();
        items
            .next()
            .map(|head| items.all(|x| x == head))
            .unwrap_or(true)
    }
}

#[cfg(test)]
mod tests {
    use super::IsUniform;
    use std::iter;

    #[test]
    fn empty_iterators_are_uniform() {
        assert!(iter::empty::<i32>().is_uniform());
    }

    #[test]
    fn single_object_iterators_are_uniform() {
        assert!(iter::once(1).is_uniform());
    }

    #[test]
    fn uniform_iterators_are_uniform() {
        assert!(iter::repeat(1).take(2).is_uniform());
    }

    #[test]
    fn non_uniform_iterators_are_not_uniform() {
        assert!(![1, 2].is_uniform());
    }
}
