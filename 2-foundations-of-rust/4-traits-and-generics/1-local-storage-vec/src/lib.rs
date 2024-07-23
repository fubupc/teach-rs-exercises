/// A growable, generic list that resides on the stack if it's small,
/// but is moved to the heap to grow larger if needed.
/// This list is generic over the items it contains as well as the
/// size of its buffer if it's on the stack.
#[derive(Debug)]
pub enum LocalStorageVec<T, const N: usize> {
    Stack { buf: [T; N], len: usize },
    Heap(Vec<T>),
}

// **Below `From` implementation is used in the tests and are therefore given. However,
// you should have a thorough look at it as they contain various new concepts.**
// This implementation is generic not only over the type `T`, but also over the
// constants `N` and 'M', allowing us to support conversions from arrays of any
// length to `LocalStorageVec`s of with any stack buffer size.
// In Rust, we call this feature 'const generics'
impl<T, const N: usize, const M: usize> From<[T; N]> for LocalStorageVec<T, M>
where
    // We require that `T` implement `Default`, in case we need to fill up our
    // stack-based array without resorting to uninitialized memory. Once
    // we are more proficient in working with uninitialized memory, we'll be
    // able to remove this bound.
    T: Default,
{
    fn from(array: [T; N]) -> Self {
        if N <= M {
            // In this case, the passed array should fit on the stack.

            // We crate an `Iterator` of the passed array,
            let mut it = array.into_iter();
            Self::Stack {
                // This is a trick for copying an array into another one that's
                // at least as long as the original, without having to create
                // default values more than strictly necessary. The `[(); M]`
                // array is zero-sized, meaning there's no cost to instantiate it.
                // The `map` call iterates over each of its items, and maps them to
                // the next item from the `array` passed to this function. If there
                // are no more items left from `array`, we insert the default specified
                // for `T`
                buf: [(); M].map(|_| it.next().unwrap_or_default()),
                // The length of the buffer on stack is the length of the original `array`: `N`
                len: N,
            }
        } else {
            // If the passed array does not fit, we'll resort to moving it to the heap instead
            Self::Heap(Vec::from(array))
        }
    }
}

impl<T, const N: usize> From<Vec<T>> for LocalStorageVec<T, N> {
    fn from(value: Vec<T>) -> Self {
        Self::Heap(value)
    }
}

// Mixed:
// - Part C: impl LocalStorageVec ⭐⭐
// - Part G: Removing bounds ⭐⭐
impl<T, const N: usize> AsRef<[T]> for LocalStorageVec<T, N> {
    fn as_ref(&self) -> &[T] {
        match self {
            LocalStorageVec::Stack { buf, len } => &buf[..*len],
            LocalStorageVec::Heap(data) => data.as_slice(),
        }
    }
}

impl<T, const N: usize> AsMut<[T]> for LocalStorageVec<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        match self {
            LocalStorageVec::Stack { buf, len } => &mut buf[..*len],
            LocalStorageVec::Heap(data) => data.as_mut_slice(),
        }
    }
}

impl<T, const N: usize> LocalStorageVec<T, N>
where
    T: Default,
{
    fn new() -> Self {
        LocalStorageVec::Stack {
            buf: [(); N].map(|_| T::default()),
            len: 0,
        }
    }

    fn push(&mut self, e: T) {
        match self {
            LocalStorageVec::Stack { buf, len } => {
                if *len == N {
                    let mut data: Vec<T> = Vec::with_capacity(N + 1);
                    for i in 0..N {
                        data.push(std::mem::take(&mut buf[i]));
                    }
                    data.push(e);
                    *self = LocalStorageVec::Heap(data)
                } else {
                    buf[*len] = e;
                    *len += 1;
                }
            }
            LocalStorageVec::Heap(data) => data.push(e),
        }
    }

    fn pop(&mut self) -> Option<T> {
        match self {
            LocalStorageVec::Stack { buf, len } => {
                if *len == 0 {
                    None
                } else {
                    let v = std::mem::take(&mut buf[*len - 1]);
                    *len -= 1;
                    Some(v)
                }
            }
            LocalStorageVec::Heap(data) => data.pop(),
        }
    }

    fn insert(&mut self, index: usize, e: T) {
        match self {
            LocalStorageVec::Stack { buf, len } => {
                if index > *len {
                    panic!("insertion index {index} should be <= len {len}");
                }
                if *len == N {
                    let mut data: Vec<T> = Vec::with_capacity(N + 1);
                    for i in 0..index {
                        data.push(std::mem::take(&mut buf[i]));
                    }
                    data.push(e);
                    for i in index..N {
                        data.push(std::mem::take(&mut buf[i]));
                    }
                    *self = LocalStorageVec::Heap(data);
                } else {
                    for i in (index..*len).rev() {
                        buf[i + 1] = std::mem::take(&mut buf[i]);
                    }
                    buf[index] = e;
                    *len += 1;
                }
            }
            LocalStorageVec::Heap(data) => data.insert(index, e),
        }
    }

    fn remove(&mut self, index: usize) -> T {
        match self {
            LocalStorageVec::Stack { buf, len } => {
                if index >= *len {
                    panic!("remove index {index} should be < len {len}");
                }
                let e = std::mem::take(&mut buf[index]);
                for i in index..(*len - 1) {
                    buf[i] = std::mem::take(&mut buf[i + 1]);
                }
                *len -= 1;
                e
            }
            LocalStorageVec::Heap(data) => data.remove(index),
        }
    }
}

impl<T, const N: usize> LocalStorageVec<T, N> {
    fn len(&self) -> usize {
        match self {
            LocalStorageVec::Stack { len, .. } => *len,
            LocalStorageVec::Heap(data) => data.len(),
        }
    }

    fn clear(&mut self) {
        match self {
            LocalStorageVec::Stack { len, .. } => *len = 0,
            LocalStorageVec::Heap(data) => data.clear(),
        }
    }
}

// Part E: Iterator and IntoIterator ⭐⭐
pub struct LocalStorageVecIter<T, const N: usize> {
    vec: LocalStorageVec<T, N>,
    counter: usize,
}

impl<T, const N: usize> Iterator for LocalStorageVecIter<T, N>
where
    T: Default,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.vec {
            LocalStorageVec::Stack { buf, len } => {
                if self.counter == *len {
                    None
                } else {
                    let e = std::mem::take(&mut buf[self.counter]);
                    self.counter += 1;
                    Some(e)
                }
            }
            LocalStorageVec::Heap(data) => {
                if self.counter == data.len() {
                    None
                } else {
                    let e = std::mem::take(&mut data[self.counter]);
                    self.counter += 1;
                    Some(e)
                }
            }
        }
    }
}

impl<T, const N: usize> IntoIterator for LocalStorageVec<T, N>
where
    T: Default,
{
    type Item = T;

    type IntoIter = LocalStorageVecIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        LocalStorageVecIter {
            vec: self,
            counter: 0,
        }
    }
}

// Part F: Index ⭐⭐
// impl<T, const N: usize> std::ops::Index<usize> for LocalStorageVec<T, N> {
//     type Output = T;

//     fn index(&self, index: usize) -> &Self::Output {
//         self.as_ref().index(index)
//     }
// }
// impl<T, const N: usize> std::ops::Index<std::ops::RangeTo<usize>> for LocalStorageVec<T, N> {
//     type Output = [T];

//     fn index(&self, index: std::ops::RangeTo<usize>) -> &Self::Output {
//         self.as_ref().index(index)
//     }
// }
// impl<T, const N: usize> std::ops::Index<std::ops::RangeFrom<usize>> for LocalStorageVec<T, N> {
//     type Output = [T];

//     fn index(&self, index: std::ops::RangeFrom<usize>) -> &Self::Output {
//         self.as_ref().index(index)
//     }
// }
// impl<T, const N: usize> std::ops::Index<std::ops::Range<usize>> for LocalStorageVec<T, N> {
//     type Output = [T];

//     fn index(&self, index: std::ops::Range<usize>) -> &Self::Output {
//         self.as_ref().index(index)
//     }
// }

// Part H: Borrowing Iterator ⭐⭐⭐
struct LocalStorageVecBorrowIter<'a, T, const N: usize> {
    vec: &'a LocalStorageVec<T, N>,
    counter: usize,
}
impl<'a, T, const N: usize> Iterator for LocalStorageVecBorrowIter<'a, T, N> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.vec {
            LocalStorageVec::Stack { buf, len } => {
                if self.counter == *len {
                    None
                } else {
                    let e = &buf[self.counter];
                    self.counter += 1;
                    Some(e)
                }
            }
            LocalStorageVec::Heap(data) => {
                if self.counter == data.len() {
                    None
                } else {
                    let e = &data[self.counter];
                    self.counter += 1;
                    Some(e)
                }
            }
        }
    }
}

impl<T, const N: usize> LocalStorageVec<T, N> {
    fn iter(&self) -> LocalStorageVecBorrowIter<'_, T, N> {
        LocalStorageVecBorrowIter {
            vec: self,
            counter: 0,
        }
    }

    // Another solution
    // fn iter<'a>(&'a self) -> std::slice::Iter<'a, T> {
    //     self.as_ref().iter()
    // }
}

// Part I: Generic Index
// pub trait LocalStorageVecIndex {}
// impl LocalStorageVecIndex for usize {}
// impl LocalStorageVecIndex for std::ops::RangeTo<usize> {}
// impl LocalStorageVecIndex for std::ops::RangeFrom<usize> {}
// impl LocalStorageVecIndex for std::ops::Range<usize> {}

impl<T, I, const N: usize> std::ops::Index<I> for LocalStorageVec<T, N>
where
    // I: LocalStorageVecIndex,
    [T]: std::ops::Index<I>,
{
    type Output = <[T] as std::ops::Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.as_ref().index(index)
    }
}

// Bonus: Another solution
// impl<T, const N: usize, I> std::ops::Index<I> for LocalStorageVec<T, N>
// where
//     I: std::slice::SliceIndex<[T]>,
// {
//     type Output = I::Output;

//     fn index(&self, index: I) -> &Self::Output {
//         self.as_ref().index(index)
//     }
// }

// Part J: Deref and DerefMut ⭐⭐⭐⭐
// Question:
// Replacing the implementation of deref with self.as_ref() results in a stack overflow when running an unoptimized version. Why? (Hint: deref coercion)
// TODO: Actually self.as_ref() does NOT results in a stack overflow. 
impl<T, const N: usize> std::ops::Deref for LocalStorageVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // <Self as AsRef<[T]>>::as_ref(self)
        self.as_ref()
    }
}

impl<T, const N: usize> std::ops::DerefMut for LocalStorageVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // <Self as AsMut<[T]>>::as_mut(self)
        self.as_mut()
    }
}

#[cfg(test)]
mod test {
    use crate::LocalStorageVec;

    #[test]
    // Don't remove the #[ignore] attribute or your tests will take forever!
    #[ignore = "This test is just to validate the definition of `LocalStorageVec`. If it compiles, all is OK"]
    #[allow(unreachable_code, unused_variables)]
    fn it_compiles() {
        // Here's a trick to 'initialize' a type while not actually
        // creating a value: an infinite `loop` expression diverges
        // and evaluates to the 'never type' `!`, which, as is can never
        // actually be instantiated, coerces to any other type.
        // Some other ways of diverging are by calling the `panic!` or the `todo!`
        // macros.
        // More info:
        // - https://doc.rust-lang.org/rust-by-example/fn/diverging.html
        // - https://doc.rust-lang.org/reference/expressions/loop-expr.html#infinite-loops
        let vec: LocalStorageVec<u32, 10> = loop {};
        match vec {
            LocalStorageVec::Stack { buf, len } => {
                let _buf: [u32; 10] = buf;
                let _len: usize = len;
            }
            LocalStorageVec::Heap(v) => {
                let _v: Vec<u32> = v;
            }
        }
    }

    // Uncomment me for part B
    #[test]
    fn it_from_vecs() {
        // The `vec!` macro creates a `Vec<T>` in a way that resembles
        // array-initialization syntax.
        let vec: LocalStorageVec<usize, 10> = LocalStorageVec::from(vec![1, 2, 3]);
        // Assert that the call to `from` indeed yields a `Heap` variant
        assert!(matches!(vec, LocalStorageVec::Heap(_)));

        let vec: LocalStorageVec<usize, 2> = LocalStorageVec::from(vec![1, 2, 3]);

        assert!(matches!(vec, LocalStorageVec::Heap(_)));
    }

    // Uncomment me for part C
    #[test]
    fn it_as_refs() {
        let vec: LocalStorageVec<i32, 256> = LocalStorageVec::from([0; 128]);
        let slice: &[i32] = vec.as_ref();
        assert!(slice.len() == 128);
        let vec: LocalStorageVec<i32, 32> = LocalStorageVec::from([0; 128]);
        let slice: &[i32] = vec.as_ref();
        assert!(slice.len() == 128);

        let mut vec: LocalStorageVec<i32, 256> = LocalStorageVec::from([0; 128]);
        let slice_mut: &[i32] = vec.as_mut();
        assert!(slice_mut.len() == 128);
        let mut vec: LocalStorageVec<i32, 32> = LocalStorageVec::from([0; 128]);
        let slice_mut: &[i32] = vec.as_mut();
        assert!(slice_mut.len() == 128);
    }

    // Uncomment me for part D
    #[test]
    fn it_constructs() {
        let vec: LocalStorageVec<usize, 10> = LocalStorageVec::new();
        // Assert that the call to `new` indeed yields a `Stack` variant with zero length
        assert!(matches!(vec, LocalStorageVec::Stack { buf: _, len: 0 }));
    }

    // Uncomment me for part D
    #[test]
    fn it_lens() {
        let vec: LocalStorageVec<_, 3> = LocalStorageVec::from([0, 1, 2]);
        assert_eq!(vec.len(), 3);
        let vec: LocalStorageVec<_, 2> = LocalStorageVec::from([0, 1, 2]);
        assert_eq!(vec.len(), 3);
    }

    // Uncomment me for part D
    #[test]
    fn it_pushes() {
        let mut vec: LocalStorageVec<_, 128> = LocalStorageVec::new();
        for value in 0..128 {
            vec.push(value);
        }
        assert!(matches!(vec, LocalStorageVec::Stack { len: 128, .. }));
        for value in 128..256 {
            vec.push(value);
        }
        assert!(matches!(vec, LocalStorageVec::Heap(v) if v.len() == 256))
    }

    // Uncomment me for part D
    #[test]
    fn it_pops() {
        let mut vec: LocalStorageVec<_, 128> = LocalStorageVec::from([0; 128]);
        for _ in 0..128 {
            assert_eq!(vec.pop(), Some(0))
        }
        assert_eq!(vec.pop(), None);

        let mut vec: LocalStorageVec<_, 128> = LocalStorageVec::from([0; 256]);
        for _ in 0..256 {
            assert_eq!(vec.pop(), Some(0))
        }
        assert_eq!(vec.pop(), None);

        let mut vec: LocalStorageVec<_, 128> = LocalStorageVec::from(vec![0; 256]);
        for _ in 0..256 {
            assert_eq!(vec.pop(), Some(0))
        }
        assert_eq!(vec.pop(), None);
    }

    // Uncomment me for part D
    #[test]
    fn it_inserts() {
        let mut vec: LocalStorageVec<_, 4> = LocalStorageVec::from([0, 1, 2]);
        vec.insert(1, 3);
        assert!(matches!(
            vec,
            LocalStorageVec::Stack {
                buf: [0, 3, 1, 2],
                len: 4
            }
        ));

        let mut vec: LocalStorageVec<_, 4> = LocalStorageVec::from([0, 1, 2, 3]);
        vec.insert(1, 3);
        assert!(matches!(vec, LocalStorageVec::Heap { .. }));
        assert_eq!(vec.as_ref(), &[0, 3, 1, 2, 3]);

        let mut vec: LocalStorageVec<_, 4> = LocalStorageVec::from([0, 1, 2, 3, 4]);
        vec.insert(1, 3);
        assert!(matches!(vec, LocalStorageVec::Heap { .. }));
        assert_eq!(vec.as_ref(), &[0, 3, 1, 2, 3, 4])
    }

    // Uncomment me for part D
    #[test]
    fn it_removes() {
        let mut vec: LocalStorageVec<_, 4> = LocalStorageVec::from([0, 1, 2]);
        let elem = vec.remove(1);
        dbg!(&vec);
        assert!(matches!(
            vec,
            LocalStorageVec::Stack {
                buf: [0, 2, _, _],
                len: 2
            }
        ));
        assert_eq!(elem, 1);

        let mut vec: LocalStorageVec<_, 2> = LocalStorageVec::from([0, 1, 2]);
        let elem = vec.remove(1);
        assert!(matches!(vec, LocalStorageVec::Heap(..)));
        assert_eq!(vec.as_ref(), &[0, 2]);
        assert_eq!(elem, 1);
    }

    // Uncomment me for part D
    #[test]
    fn it_clears() {
        let mut vec: LocalStorageVec<_, 10> = LocalStorageVec::from([0, 1, 2, 3]);
        assert!(matches!(vec, LocalStorageVec::Stack { buf: _, len: 4 }));
        vec.clear();
        assert_eq!(vec.len(), 0);

        let mut vec: LocalStorageVec<_, 3> = LocalStorageVec::from([0, 1, 2, 3]);
        assert!(matches!(vec, LocalStorageVec::Heap(_)));
        vec.clear();
        assert_eq!(vec.len(), 0);
    }

    // Uncomment me for part E
    #[test]
    fn it_iters() {
        let vec: LocalStorageVec<_, 128> = LocalStorageVec::from([0; 32]);
        let mut iter = vec.into_iter();
        for item in &mut iter {
            assert_eq!(item, 0);
        }
        assert_eq!(iter.next(), None);

        let vec: LocalStorageVec<_, 128> = LocalStorageVec::from(vec![0; 128]);
        let mut iter = vec.into_iter();
        for item in &mut iter {
            assert_eq!(item, 0);
        }
        assert_eq!(iter.next(), None);
    }

    // Uncomment me for part F
    #[test]
    fn it_indexes() {
        let vec: LocalStorageVec<i32, 10> = LocalStorageVec::from([0, 1, 2, 3, 4, 5]);
        assert_eq!(vec[1], 1);
        assert_eq!(vec[..2], [0, 1]);
        assert_eq!(vec[4..], [4, 5]);
        assert_eq!(vec[1..3], [1, 2]);
    }

    // Uncomment me for part H
    #[test]
    fn it_borrowing_iters() {
        let vec: LocalStorageVec<String, 10> = LocalStorageVec::from([
            "0".to_owned(),
            "1".to_owned(),
            "2".to_owned(),
            "3".to_owned(),
            "4".to_owned(),
            "5".to_owned(),
        ]);
        let iter = vec.iter();
        for _ in iter {}
        // This requires the `vec` not to be consumed by the call to `iter()`
        drop(vec);
    }

    // Uncomment me for part J
    #[test]
    fn it_derefs() {
        use std::ops::{Deref, DerefMut};
        let vec: LocalStorageVec<_, 128> = LocalStorageVec::from([0; 128]);
        // `chunks` is a method that's defined for slices `[T]`, that we can use thanks to `Deref`
        let chunks = vec.chunks(4);
        let slice: &[_] = vec.deref();
    
        let mut vec: LocalStorageVec<_, 128> = LocalStorageVec::from([0; 128]);
        let chunks = vec.chunks_mut(4);
        let slice: &mut [_] = vec.deref_mut();
    }
}
