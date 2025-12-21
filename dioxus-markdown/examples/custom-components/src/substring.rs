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

    fn read(&self) -> String {
        let str = self.s.read();
        str[self.range.clone()].to_string()
    }
}

/// An updatable substring, and cached value read from it.
struct ParsedSubString<T> {
    /// On write, the substring is updated.
    sub: SubString,
    /// On read, current is used, which is typically (but not necessarily) parsed from the substring.
    current: T,
}

impl<T> ReadWrite<T> for ParsedSubString<T>
where
    T: Clone + ToString,
{
    fn read_value(&self) -> T {
        self.current.clone()
    }

    fn write_value(&self, t: T) {
        let s = t.to_string();
        self.sub.write(&s);
    }
}

trait ReadWrite<T> {
    fn read_value(&self) -> T;
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
    pub fn read_value(&self) -> T {
        self.content.read_value()
    }

    pub fn write_value(&self, t: T) {
        self.content.write_value(t);
    }
}

impl<T: Clone + FromStr + Display + 'static> ReadWriteBox<T> {
    pub fn from_sub_string(s: Signal<String>, range: Range<usize>) -> Result<Self, T::Err> {
        let sub = { SubString { s, range } };
        let current = T::from_str(&sub.read())?;
        let inner = ParsedSubString { current, sub };
        Ok(ReadWriteBox {
            content: Rc::new(inner),
        })
    }
}

impl<T: std::ops::Sub<T, Output = T> + Clone> std::ops::SubAssign<T> for ReadWriteBox<T> {
    fn sub_assign(&mut self, rhs: T) {
        self.write_value(self.read_value() - rhs);
    }
}

impl<T: std::ops::Add<T, Output = T> + Clone> std::ops::AddAssign<T> for ReadWriteBox<T> {
    fn add_assign(&mut self, rhs: T) {
        self.write_value(self.read_value() + rhs);
    }
}

impl<T: Display> Display for ReadWriteBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value().fmt(f)
    }
}
