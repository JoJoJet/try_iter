use std::iter::FromIterator;

pub trait TryIterator: Sized
where
    Self: Iterator<Item = Result<Self::Ok, Self::Err>>,
{
    type Ok;
    type Err;

    #[inline]
    fn try_map<F, T>(self, f: F) -> TryMap<Self, F>
    where
        F: FnMut(Self::Ok) -> T,
    {
        TryMap { iter: self, f }
    }

    #[inline]
    fn try_flat_map<F, T, E>(self, f: F) -> TryFlatMap<Self, F>
    where
        F: FnMut(Self::Ok) -> Result<T, E>,
    {
        TryFlatMap { iter: self, f }
    }

    #[inline]
    fn take_ok(self) -> TakeOk<Self> {
        TakeOk {
            iter: self,
            flag: false,
        }
    }

    #[inline]
    fn filter_ok(self) -> FilterOk<Self> {
        FilterOk(self)
    }

    #[inline]
    fn try_collect<B>(self) -> Result<B, Self::Err>
    where
        B: FromIterator<Self::Ok>,
    {
        FromIterator::from_iter(self)
    }
}

impl<I, T, E> TryIterator for I
where
    I: Iterator<Item = Result<T, E>>,
{
    type Ok = T;
    type Err = E;
}

pub struct TryMap<I, F> {
    iter: I,
    f: F,
}

impl<I, F, T> Iterator for TryMap<I, F>
where
    I: TryIterator,
    F: FnMut(I::Ok) -> T,
{
    type Item = Result<T, I::Err>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|r| r.map(&mut self.f))
    }
}

pub struct TryFlatMap<I, F> {
    iter: I,
    f: F,
}

impl<I, F, T, E> Iterator for TryFlatMap<I, F>
where
    I: TryIterator,
    F: FnMut(I::Ok) -> Result<T, E>,
    E: From<I::Err>,
{
    type Item = Result<T, E>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|r| r.map_err(From::from).and_then(&mut self.f))
    }
}

pub struct TakeOk<I> {
    iter: I,
    flag: bool,
}

impl<I: TryIterator> Iterator for TakeOk<I> {
    type Item = I::Ok;
    fn next(&mut self) -> Option<Self::Item> {
        if self.flag {
            None
        } else {
            match self.iter.next()? {
                Ok(x) => Some(x),
                Err(_) => {
                    self.flag = true;
                    None
                }
            }
        }
    }
}

pub struct FilterOk<I>(I);

impl<I: TryIterator> Iterator for FilterOk<I> {
    type Item = I::Ok;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.find_map(Result::ok)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let s = vec!["1", "2", "3", "four", "5"];
        let mut i = s.into_iter().map(str::parse::<i32>).try_map(|n| n + 1);
        assert_eq!(i.next(), Some(Ok(2)));
        assert_eq!(i.next(), Some(Ok(3)));
        assert_eq!(i.next(), Some(Ok(4)));
        assert!(i.next().unwrap().is_err());
        assert_eq!(i.next(), Some(Ok(6)));

        let s = vec!["1", "2", "3", "4"];
        let v: Vec<_> = s.into_iter().map(str::parse::<i32>).try_collect().unwrap();
        assert_eq!(v, vec![1, 2, 3, 4]);

        let s = ["1", "2", "three", "4"];
        let v = s.iter().map(|s| s.parse::<i32>()).try_collect::<Vec<_>>();
        assert!(v.is_err());

        let v: Vec<_> = s.iter().map(|s| s.parse::<i32>()).take_ok().collect();
        assert_eq!(v, vec![1, 2]);

        let v: Vec<_> = s.iter().map(|s| s.parse::<i32>()).filter_ok().collect();
        assert_eq!(v, vec![1, 2, 4]);
    }
}
