//! Composable external pagination.
//!
//! If you’ve found yourself with a collection of some kind, and needed to perform an operation on the elements of said collection,
//! you’ll often use iterators. Paginators are a similar concept, but they allow you to revisit previous elements.
//!
//! Think of a paginator as a vynil disc and a needle head. You can move the needle forward, but also backwards.
//!
//! ## Paginator
//!
//! The heart and soul of this module is the [Paginator] trait. The core of [Paginator] looks like this:
//! ```rust
//! pub trait Paginator {
//!     type Item;
//!
//!     fn next(&mut self) -> Option<Self::Item>;
//!     fn previous(&mut self) -> Option<Self::Item>;
//! }
//! ```
//! A paginator has two methods, next and previous, which when called, return an `Option<Item>`.
//! Calling next will return Some(Item) as long as there are elements, and once they’ve all been exhausted,
//! will return None to indicate that iteration is finished _in this direction_. Calling previous yields the elements you saw before.
//!
//! [Paginator]'s full definition includes a number of other methods as well,
//! but they are default methods, built on top of next, and so you get them for free.
//!
//! ## The two forms of pagination
//!
//! There are two common methods which can create paginators from a collection:
//!
//! - `paginate()`, which paginates over `&T`.
//! - ~~`paginate_mut()`, which paginates over `&mut T`.~~ (not available because of lifetime shenanigans...)
//!
//! Various things in the standard library and in this crate may implement one or more of the two, where appropriate.
//!
//! There is no scenario in which you'd paginate over a `T` directly,
//! but during the pagination you can copy and clone the elements.
//!
//! ## Implementing Paginator
//!
//! Implementing paginator is a similar experience to implementing [Iterator].
//! I recommend reading its documentation page to understand State, Adapters, Infinity, etc.
//! 
//! ## Laziness
//! 
//! Paginators (and paginator adapters) are lazy. This means that just creating a paginator doesn’t do a whole lot.
//! Nothing really happens until you call next. This is sometimes a source of confusion when creating a paginator solely for its side effects.
//! For example, the map method calls a closure on each element it iterates over:
//! 
//! ```rust
//! let v = vec![1, 2, 3, 4, 5];
//! v.paginate().map(|x| println!("{x}"));
//! ```
//! 
//! This will not print any values, as we only created an iterator, rather than using it.
//! The compiler will warn us about this kind of behavior:
//! 
//! ```warning: unused result that must be used: paginators are lazy and do nothing unless consumed```

use adapters::{Chain, ChainState, Enumerate, Map};

pub mod adapters;

/// The core paginator trait.
#[must_use = "paginators are lazy and do nothing unless consumed"]
pub trait Paginator {
    /// The type of element this paginator yields.
    type Item;

    /// Returns the next element or `None` if you've reached the end.
    fn next(&mut self) -> Option<Self::Item>;

    /// Returns the next element or `None` if you've reached the start.
    fn previous(&mut self) -> Option<Self::Item>;

    /// Adapts this paginator to one that modifies the element before visualizing it!
    fn map<F, Output>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Item) -> Output,
        Self: Sized,
    {
        Map { inner: self, f }
    }

    /// Adapts this paginator to one that also yields the index of the current element.
    #[inline]
    fn enumerate(self) -> Enumerate<Self>
    where
        Self: Sized,
    {
        Enumerate {
            index: 0,
            inner: self,
        }
    }

    /// Returns a new paginator that yields elements from B after A's elements run out.
    #[inline]
    fn chain<B>(self, other: B) -> Chain<Self, B>
    where
        Self: Sized,
        B: Paginator,
    {
        Chain {
            disjunctor: ChainState::First,
            inner_a: self,
            inner_b: other,
        }
    }
}

// #[test]
// fn test_chain_paginator2() {
//     struct Vecs(pub Vec<i32>, pub Vec<i32>);
//     let vecs = Vecs(vec![0, 1], vec![2, 3]);

//     let pa = vecs.0.paginate();
//     let pb = vecs.1.paginate();
//     let mut chain = Paginator::chain(pa, pb);
//     if let Some(num) = chain.next() {}
// }

/// Trait for conversion into a temporary paginator.
pub trait Paginate<'pag> {
    type Paginator: Paginator;

    fn paginate(&'pag self) -> Self::Paginator;
}

/// Returns a paginator with a single element.
pub fn once<T>(element: &T) -> Once<T> {
    Once {
        position: 0,
        element,
    }
}

/// Paginator that holds a single element.
pub struct Once<'pag, A> {
    position: usize,
    element: &'pag A,
}

impl<'pag, A: 'pag> Paginator for Once<'pag, A> {
    type Item = &'pag A;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.position == 0 {
            self.position += 1;
            Some(self.element)
        } else {
            None
        }
    }

    #[inline]
    fn previous(&mut self) -> Option<Self::Item> {
        if self.position == 1 {
            self.position -= 1;
            Some(self.element)
        } else {
            None
        }
    }
}

#[test]
fn test_once_paginator() {
    let mut o = once(&5);
    assert_eq!(o.previous(), None);
    assert_eq!(o.next(), Some(&5));
    assert_eq!(o.next(), None);
    assert_eq!(o.previous(), Some(&5));
    assert_eq!(o.previous(), None);
}

/// A paginator that views the elements of a [Vec].
pub struct VecPag<'pag, A> {
    pub items: &'pag Vec<A>,
    pub index: usize,
}

impl<'pag, A: 'pag> Paginator for VecPag<'pag, A> {
    type Item = &'pag A;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.items.get(self.index).inspect(|element| {
            self.index += 1;
        })
    }

    #[inline]
    fn previous(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            return None;
        };

        self.items.get(self.index - 1).inspect(|element| {
            self.index -= 1;
        })
    }
}

impl<'pag, A: 'pag> Paginate<'pag> for Vec<A> {
    type Paginator = VecPag<'pag, A>;

    fn paginate(&'pag self) -> Self::Paginator {
        VecPag {
            items: self,
            index: 0,
        }
    }
}

#[test]
fn test_vec_paginator() {
    let items = vec![0, 1, 2, 3];
    let mut pag = items.paginate();

    assert_eq!(pag.next(), Some(&0));
    assert_eq!(pag.next(), Some(&1));
    assert_eq!(pag.next(), Some(&2));
    assert_eq!(pag.next(), Some(&3));
    assert_eq!(pag.next(), None);
    assert_eq!(pag.previous(), Some(&3));
    assert_eq!(pag.previous(), Some(&2));
    assert_eq!(pag.previous(), Some(&1));
    assert_eq!(pag.previous(), Some(&0));
    assert_eq!(pag.previous(), None);
}
