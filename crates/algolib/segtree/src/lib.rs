use std::{
    fmt::{self, Debug, Formatter},
    ops::{Bound, Deref, DerefMut, Drop, Index, Range, RangeBounds},
    slice::SliceIndex,
};

pub struct Segtree<T, Op, Ideneity> {
    len: usize,
    table: Box<[T]>,
    op: Op,
    identity: Ideneity,
}
impl<T, Op, Identity> Segtree<T, Op, Identity>
where
    T: Clone,
    Op: Fn(T, T) -> T,
    Identity: Fn() -> T,
{
    pub fn new(slice: &[T], op: Op, identity: Identity) -> Self {
        let len = slice.len();
        let mut table = slice.to_vec();
        table.extend(slice.iter().cloned());
        let mut table = table.into_boxed_slice();
        (1..len)
            .rev()
            .for_each(|i| table[i] = op(table[2 * i].clone(), table[2 * i + 1].clone()));
        Self {
            len,
            table,
            op,
            identity,
        }
    }
    pub fn as_slice(&self) -> &[T] {
        &self.table[self.len..]
    }
    pub fn entry(&mut self, index: usize) -> Entry<'_, T, Op, Identity> {
        Entry { seg: self, index }
    }
    pub fn fold(&self, range: impl RangeBounds<usize>) -> T {
        let Range { mut start, mut end } = open(range, self.len);
        start += self.len;
        end += self.len;
        let mut fl = (self.identity)();
        let mut fr = (self.identity)();
        while start != end {
            if start % 2 == 1 {
                fl = (self.op)(fl, self.table[start].clone());
                start += 1;
            }
            if end % 2 == 1 {
                end -= 1;
                fr = (self.op)(self.table[end].clone(), fr);
            }
            start /= 2;
            end /= 2;
        }
        (self.op)(fl, fr)
    }
}

fn open(range: impl RangeBounds<usize>, len: usize) -> Range<usize> {
    (match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Included(&x) => x,
        Bound::Excluded(&x) => x + 1,
    })..match range.end_bound() {
        Bound::Excluded(&x) => x,
        Bound::Included(&x) => x + 1,
        Bound::Unbounded => len,
    }
}

impl<T: Debug, Op, Identity> Debug for Segtree<T, Op, Identity>
where
    T: Clone,
    Op: Fn(T, T) -> T,
    Identity: Fn() -> T,
{
    fn fmt(&self, w: &mut Formatter<'_>) -> fmt::Result {
        w.debug_tuple("Segtree")
            .field(&&self.table[self.len..])
            .finish()
    }
}

impl<I: SliceIndex<[T]>, T, Op, Identity> Index<I> for Segtree<T, Op, Identity>
where
    T: Clone,
    Op: Fn(T, T) -> T,
    Identity: Fn() -> T,
{
    type Output = I::Output;
    fn index(&self, index: I) -> &I::Output {
        &self.as_slice()[index]
    }
}

pub struct Entry<'a, T, Op, Identity>
where
    T: Clone,
    Op: Fn(T, T) -> T,
    Identity: Fn() -> T,
{
    seg: &'a mut Segtree<T, Op, Identity>,
    index: usize,
}

impl<'a, T, Op, Identity> Drop for Entry<'a, T, Op, Identity>
where
    T: Clone,
    Op: Fn(T, T) -> T,
    Identity: Fn() -> T,
{
    fn drop(&mut self) {
        let mut index = self.index + self.seg.len;
        while index != 0 {
            index /= 2;
            self.seg.table[index] = (self.seg.op)(
                self.seg.table[index * 2].clone(),
                self.seg.table[index * 2 + 1].clone(),
            );
        }
    }
}

impl<'a, T, Op, Identity> Deref for Entry<'a, T, Op, Identity>
where
    T: Clone,
    Op: Fn(T, T) -> T,
    Identity: Fn() -> T,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.seg[self.index.clone()]
    }
}

impl<'a, T, Op, Identity> DerefMut for Entry<'a, T, Op, Identity>
where
    T: Clone,
    Op: Fn(T, T) -> T,
    Identity: Fn() -> T,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        let index = self.index;
        let len = self.seg.len;
        &mut (self.seg.table[len..])[index]
    }
}

#[cfg(test)]
mod tests {
    use {
        super::Segtree,
        rand::{distributions::Alphanumeric, prelude::*},
        randtools::SubRange,
        std::iter,
    };

    #[test]
    fn test_index() {
        let seg = Segtree::new(&[0, 1], |x, _y| x, || 0);
        assert_eq!(seg[0], 0);
        assert_eq!(seg[1], 1);
        assert_eq!(&seg[..], &[0, 1]);
    }

    #[test]
    fn test_as_slice() {
        let seg = Segtree::new(&[0, 1], |x, _y| x, || 0);
        assert_eq!(seg.as_slice(), &[0, 1]);
    }

    #[test]
    fn test_entry() {
        let mut seg = Segtree::new(&[0, 1], |x, _y| x, || 0);
        *seg.entry(0) = 10;
        *seg.entry(1) = 11;
        assert_eq!(seg.as_slice(), &[10, 11]);
        *seg.entry(0) = 20;
        *seg.entry(1) = 21;
        assert_eq!(seg.as_slice(), &[20, 21]);
    }

    #[test]
    fn test_strcat() {
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..20 {
            let n = rng.gen_range(1, 40);
            let mut a = iter::repeat_with(|| iter::once(rng.sample(Alphanumeric)).collect())
                .take(n)
                .collect::<Vec<_>>();
            let mut seg = Segtree::new(&a, strcat, String::new);
            println!("a = {:?}", &a);
            println!("seg = {:?}", &seg);
            for _ in 0..200 {
                match rng.gen_range(0, 2) {
                    0 => {
                        let i = rng.gen_range(0, n);
                        let s = iter::once(rng.sample(Alphanumeric)).collect::<String>();
                        a[i] = s.clone();
                        *seg.entry(i) = s;
                    }
                    1 => {
                        let range = rng.sample(SubRange(0..n));
                        let result = seg.fold(range.clone());
                        let expected = a[range].iter().cloned().fold(String::new(), strcat);
                        assert_eq!(result, expected);
                    }
                    _ => panic!(),
                }
            }
        }
        fn strcat(s: String, t: String) -> String {
            s.chars().chain(t.chars()).collect::<String>()
        }
    }
}
