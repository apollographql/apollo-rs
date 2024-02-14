use arbitrary::Result;

pub(crate) trait UnstructuredExt {
    fn arbitrary_vec<T, C: Fn(&mut Unstructured) -> Result<T>>(
        &mut self,
        min: usize,
        max: usize,
        callback: C,
    ) -> Result<Vec<T>>;
}
impl<'a> UnstructuredExt for Unstructured<'a> {
    fn arbitrary_vec<T, C: Fn(&mut Unstructured) -> Result<T>>(
        &mut self,
        min: usize,
        max: usize,
        callback: C,
    ) -> Result<Vec<T>> {
        let count = self.int_in_range(min..=max)?;
        let mut results = Vec::with_capacity(count);
        for _ in 0..count {
            results.push(callback(self)?);
        }
        Ok(results)
    }
}

pub(crate) trait UnstructuredOption: Sized {
    fn optional(self, u: &mut Unstructured) -> Result<Option<Self>>;
}

impl<T> UnstructuredOption for T {
    fn optional(self, u: &mut Unstructured) -> Result<Option<T>> {
        if u.arbitrary()? {
            Ok(Some(self))
        } else {
            Ok(None)
        }
    }
}

struct Unstructured<'a> {
    u: &'a mut Unstructured<'a>,
}
