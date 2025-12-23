//! Utilities for working with mutable substrings in Dioxus.
//!
//! There is probably a better way to handle editable projections of derived data in Dioxus using Stores and Lens,
//! but this works well enough for now.

use dioxus::signals::{ReadableExt, Signal, WritableExt};
use std::fmt::Display;
use std::ops::Range;
use std::rc::Rc;
use std::str::FromStr;

/// Like a signal for part of a string.
struct SubString {
    s: Signal<String>,
    range: Range<usize>,
}

impl SubString {
    fn write(&self, sub: &str) {
        let mut s2 = self.s;
        let mut str = s2.write();
        str.replace_range(self.range.clone(), sub);
    }

    /// Read the substring content.
    ///
    /// TODO:
    /// If the lifetimes could be worked out, having this be `read` and return
    /// &str or `impl Deref<str>` would probably be better, but for now this works fine.
    fn map<Out>(&self, f: impl Fn(&str) -> Out) -> Out {
        let str = self.s.read();
        f(&str[self.range.clone()])
    }
}

/// An updatable substring, and cached value read from it.
struct ParsedSubString<T> {
    /// On write, the substring is updated.
    sub: SubString,
    /// On read, current is used, which is typically (but not necessarily) parsed from the substring.
    current: T,
}

impl<T: ToString> ReadWrite<T> for ParsedSubString<T> {
    fn read_value(&self) -> &T {
        &self.current
    }

    fn write_value(&self, t: T) {
        let s = t.to_string();
        self.sub.write(&s);
    }
}

/// Like a signal, but supports outputting derived data
/// so long writes can be transformed back to corresponding changes to the original data source.
trait ReadWrite<T> {
    fn read_value(&self) -> &T;
    fn write_value(&self, t: T);
}

#[derive(Clone)]
pub struct ReadWriteBox<T> {
    content: Rc<dyn ReadWrite<T>>,
}

impl<T> PartialEq for ReadWriteBox<T> {
    fn eq(&self, other: &Self) -> bool {
        // TODO: this is likely not the best comparison.
        Rc::ptr_eq(&self.content, &other.content)
    }
}

impl<T> ReadWriteBox<T> {
    pub fn read_value(&self) -> &T {
        self.content.read_value()
    }

    pub fn write_value(&self, t: T) {
        self.content.write_value(t);
    }
}

impl<T: Clone + FromStr + Display + 'static> ReadWriteBox<T> {
    pub fn from_sub_string(s: Signal<String>, range: Range<usize>) -> Result<Self, T::Err> {
        let sub = { SubString { s, range } };
        let current = sub.map(T::from_str)?;
        let inner = ParsedSubString { current, sub };
        Ok(ReadWriteBox {
            content: Rc::new(inner),
        })
    }
}

impl<T> std::ops::SubAssign<T> for ReadWriteBox<T>
where
    for<'a> &'a T: std::ops::Sub<T, Output = T>,
{
    fn sub_assign(&mut self, rhs: T) {
        self.write_value(self.read_value() - rhs);
    }
}

impl<T> std::ops::AddAssign<T> for ReadWriteBox<T>
where
    for<'a> &'a T: std::ops::Add<T, Output = T>,
{
    fn add_assign(&mut self, rhs: T) {
        self.write_value(self.read_value() + rhs);
    }
}

impl<T: Display> Display for ReadWriteBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value().fmt(f)
    }
}
