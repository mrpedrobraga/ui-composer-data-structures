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
//! ```
//! pub trait Paginator {
//!     type Item;
//! 
//!     fn next(&mut self) -> Option<Self::Item>;
//!     fn previous(&mut self) -> Option<Self::Item>;
//! }
//! ```
//! An iterator has two methods, next and previous, which when called, return an Option<Item>.
//! Calling next will return Some(Item) as long as there are elements, and once they’ve all been exhausted,
//! will return None to indicate that iteration is finished _in this direction_. Calling previous yields the elements you saw before.
//! 
//! [Paginator]'s full definition includes a number of other methods as well,
//! but they are default methods, built on top of next, and so you get them for free.
//!
//! ## The two forms of pagination
//! 
//! There are two common methods which can create iterators from a collection:
//!
//! - paginate(), which paginates over &T.
//! - paginate_mut(), which paginates over &mut T.
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

/// The core paginator trait.
pub trait Paginator<'pag> {
    // The type of element this paginator yields.
    type Item<'view> where 'pag: 'view, Self: 'view;

    // Returns the next element or `None` if you've reached the end.
    fn next<'view>(&'view mut self) -> Option<Self::Item<'view>> where 'pag: 'view;

    // Returns the next element or `None` if you've reached the start.
    fn previous<'view>(&'view mut self) -> Option<Self::Item<'view>> where 'pag: 'view;

}

/// Trait for conversion into a temporary paginator.
pub trait Paginate<'pag> {
    type Paginator: Paginator<'pag>;

    fn paginate(&'pag self) -> Self::Paginator;
}

/// Trait for conversion into a temporary mutable paginator.
pub trait PaginateMut<'pag> {
    type Paginator: Paginator<'pag>;

    fn paginate_mut(&'pag mut self) -> Self::Paginator;
}

/// Returns a paginator with a single element.
pub fn once<T>(element: &T) -> Once<T> {
    Once { position: 0, element }
}

/// Paginator that holds a single element.
pub struct Once<'pag, A> {
    position: usize,
    element: &'pag A,
}

impl<'pag, A: 'pag> Paginator<'pag> for Once<'pag, A> {
    type Item<'view> = &'pag A where 'pag: 'view, Self: 'view;

    fn next<'view>(&'view mut self) -> Option<Self::Item<'view>> where 'pag: 'view {
        if self.position == 0 {
            self.position += 1;
            Some(&self.element)
        } else {
            None
        }
    }

    fn previous<'view>(&'view mut self) -> Option<Self::Item<'view>> where 'pag: 'view {
        if self.position == 1 {
            self.position -= 1;
            Some(&self.element)
        } else {
            None
        }
    }
}

#[test]
fn test_once() {
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

impl<'pag, A: 'pag> Paginator<'pag> for VecPag<'pag, A> {
    type Item<'view> = &'pag A where 'pag: 'view, Self: 'view;

    fn next<'view>(&'view mut self) -> Option<Self::Item<'pag>> where 'pag: 'view {
        self.items.get(self.index).map(|element| {
            self.index += 1;
            element
        })
    }

    fn previous<'view>(&'view mut self) -> Option<Self::Item<'pag>> where 'pag: 'view {
        if self.index == 0 {
            return None;
        };

        self.items.get(self.index - 1).map(|element| {
            self.index -= 1;
            element
        })
    }
}

impl<'pag, A: 'pag> Paginate<'pag> for Vec<A> {
    type Paginator = VecPag<'pag, A>;
    
    fn paginate(&'pag self) -> Self::Paginator {
        VecPag { items: &self, index: 0 }
    }
}

/// A paginator that edits the elements of a [Vec].
pub struct VecPagMut<'pag, A> {
    pub items: &'pag mut Vec<A>,
    pub index: usize,
}

impl<'pag, A: 'pag> Paginator<'pag> for VecPagMut<'pag, A> {
    type Item<'view> = &'view mut A where 'pag: 'view, Self: 'view;

    fn next<'view>(&'view mut self) -> Option<Self::Item<'view>> where 'pag: 'view {
        self.items.get_mut(self.index).map(|element| {
            self.index += 1;
            element
        })
    }

    fn previous<'view>(&'view mut self) -> Option<Self::Item<'view>> where 'pag: 'view {
        if self.index == 0 {
            return None;
        };

        self.items.get_mut(self.index - 1).map(|element| {
            self.index -= 1;
            element
        })
    }
}

impl<'pag, A: 'pag> PaginateMut<'pag> for Vec<A> {
    type Paginator = VecPagMut<'pag, A>;
    
    fn paginate_mut(&'pag mut self) -> Self::Paginator {
        VecPagMut { items: self, index: 0 }
    }
}

#[test]
fn test_vec() {
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

#[test]
fn test_vec_mut() {
    let mut items = vec![0, 1, 2, 3];
    let mut pag = VecPagMut { items: &mut items, index: 0 };

    assert_eq!(pag.next(), Some(&mut 0));
    assert_eq!(pag.next(), Some(&mut 1));
    assert_eq!(pag.next(), Some(&mut 2));
    assert_eq!(pag.next(), Some(&mut 3));
    assert_eq!(pag.next(), None);
    assert_eq!(pag.previous(), Some(&mut 3));
    assert_eq!(pag.previous(), Some(&mut 2));
    assert_eq!(pag.previous(), Some(&mut 1));
    assert_eq!(pag.previous(), Some(&mut 0));
    assert_eq!(pag.previous(), None);
}
