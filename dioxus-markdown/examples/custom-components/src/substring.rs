//! Utilities for working with mutable substrings in Dioxus.
//!
//! There is probably a better way to handle editable projections of derived data in Dioxus using Stores and Lens,
//! but this works well enough for now.

use dioxus::signals::{ReadableExt, Signal, WritableExt};
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::Range;
use std::rc::Rc;
use std::str::FromStr;

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

struct ParsedSubString<T> {
    sub: SubString,
    phantom: PhantomData<T>,
}

impl<T> ReadWrite<T> for ParsedSubString<T>
where
    T: FromStr + Clone + ToString,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn read_value(&self) -> T {
        let s = self.sub.read();
        let x = T::from_str(&s);
        x.unwrap()
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

impl<T: Clone + FromStr + Display + 'static> ReadWriteBox<T>
where
    <T as FromStr>::Err: std::fmt::Debug,
{
    pub fn from_sub_string(s: Signal<String>, range: Range<usize>) -> Self {
        let inner: ParsedSubString<T> = ParsedSubString {
            phantom: PhantomData,
            sub: { SubString { s, range } },
        };
        ReadWriteBox {
            content: Rc::new(inner),
        }
    }
}

impl<T: std::ops::Sub<T, Output = T> + Clone + FromStr + Display + 'static> std::ops::SubAssign<T>
    for ReadWriteBox<T>
{
    fn sub_assign(&mut self, rhs: T) {
        self.write_value(self.read_value() - rhs);
    }
}

impl<T: std::ops::Add<T, Output = T> + Clone + FromStr + Display + 'static> std::ops::AddAssign<T>
    for ReadWriteBox<T>
{
    fn add_assign(&mut self, rhs: T) {
        self.write_value(self.read_value() + rhs);
    }
}

impl<T: std::ops::Add<T, Output = T> + Clone + FromStr + Display + 'static> Display
    for ReadWriteBox<T>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value().fmt(f)
    }
}
