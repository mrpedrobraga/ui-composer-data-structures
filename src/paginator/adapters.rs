//! This module contains adapters for [Paginator]s, which allows you to compose them together
//! into new paginators, exactly like you'd do with [Iterator]s from [std::iter].

use super::Paginator;

/// Struct created by [Paginator::map]. See that method for more information.
#[must_use = "paginators are lazy and do nothing unless consumed"]
pub struct Map<A, F> {
    pub(crate) inner: A,
    pub(crate) f: F,
}

impl<A, F, Output> Paginator for Map<A, F>
where
    A: Paginator,
    F: Fn(A::Item) -> Output,
{
    type Item = Output;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(&self.f)
    }

    #[inline]
    fn previous(&mut self) -> Option<Self::Item> {
        self.inner.previous().map(&self.f)
    }
}

#[test]
fn test_map_paginator() {
    use crate::paginator::Paginate as _;

    let a = vec![20, 30];
    let mut p = a.paginate().map(|el| el.to_string());

    assert_eq!(p.next(), Some(String::from("20")));
    assert_eq!(p.next(), Some(String::from("30")));
}

/// Struct created by [Paginator::enumerate]. See that method for more information.
#[must_use = "paginators are lazy and do nothing unless consumed"]
pub struct Enumerate<A> {
    pub(crate) index: usize,
    pub(crate) inner: A,
}

impl<A: Paginator> Paginator for Enumerate<A> {
    type Item = (usize, A::Item);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|element| {
            let old_index = self.index;
            self.index += 1;
            (old_index, element)
        })
    }

    #[inline]
    fn previous(&mut self) -> Option<Self::Item> {
        self.inner.previous().map(|element| {
            let old_index = self.index;
            self.index -= 1;
            (old_index, element)
        })
    }
}

#[test]
fn test_enumerate_paginator() {
    use crate::paginator::Paginate as _;

    let items = vec!["Hello", "World"];
    let mut o = items.paginate().enumerate();

    assert_eq!(o.next(), Some((0, &"Hello")));
    assert_eq!(o.next(), Some((1, &"World")));
}

/// Struct created by [Paginator::chain]. See that method for more information..
#[must_use = "paginators are lazy and do nothing unless consumed"]
pub struct Chain<A, B> {
    pub(crate) disjunctor: ChainState,
    pub(crate) inner_a: A,
    pub(crate) inner_b: B,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChainState {
    First,
    Second,
}

impl<'pag, A, B> Paginator for Chain<A, B>
where
    A: 'pag + Paginator,
    B: 'pag + Paginator,
    B::Item: Into<A::Item>,
{
    type Item = A::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let ChainState::First = self.disjunctor {
            let next_a = self.inner_a.next();
            if let Some(next_a) = next_a {
                return Some(next_a);
            } else {
                self.disjunctor = ChainState::Second
            }
        }

        if let ChainState::Second = self.disjunctor {
            let next_b = self.inner_b.next();
            if let Some(next_b) = next_b {
                self.disjunctor = ChainState::Second;
                return Some(next_b.into());
            }
        }

        None
    }

    #[inline]
    fn previous(&mut self) -> Option<Self::Item> {
        if let ChainState::Second = self.disjunctor {
            let previous_b = self.inner_b.previous();
            if let Some(previous_b) = previous_b {
                return Some(previous_b.into());
            } else {
                self.disjunctor = ChainState::First
            }
        }

        if let ChainState::First = self.disjunctor {
            let previous_a = self.inner_a.previous();
            if let Some(previous_a) = previous_a {
                return Some(previous_a);
            }
        }

        None
    }
}

#[test]
fn test_chain_paginator<'test>() {
    use crate::paginator::Paginate as _;

    let a = vec![0, 1];
    let b = vec![2, 3];

    let ap = a.paginate();
    let bp = b.paginate();

    let p = Box::leak(Box::new(Paginator::chain(ap, bp)));
    assert_eq!(Paginator::next(p), Some(&0));
    assert_eq!(Paginator::next(p), Some(&1));
    assert_eq!(Paginator::next(p), Some(&2));
    assert_eq!(Paginator::next(p), Some(&3));
    assert_eq!(Paginator::next(p), None);
    assert_eq!(Paginator::previous(p), Some(&3));
    assert_eq!(Paginator::previous(p), Some(&2));
    assert_eq!(Paginator::previous(p), Some(&1));
    assert_eq!(Paginator::previous(p), Some(&0));
    assert_eq!(Paginator::previous(p), None);
}
