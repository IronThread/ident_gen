//! Little crate exporting a generator of identifiers.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use core::{
    fmt::{self, Debug, Display, Formatter},
    ops::{Deref, DerefMut},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Identifier generator,it can generate all possible combinations for a table for all possible lengths
/// in order,the default table it's [`DEFAULT_TABLE`].
pub struct IdentGen<'a> {
    pub ident: String,
    table: &'a [char],
}

/// All characters that are in the lower `snake_case` ascii standard.
pub const DEFAULT_TABLE: &'static [char] = &[
    'a', 'b', 'c', 'd', 'f', 'e', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'z', '_',
];

/// All characters that are in the upper `SNAKE_CASE` ascii standard.
pub const UPPER_SNAKE: &'static [char] = &[
    'A', 'B', 'C', 'D', 'F', 'E', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Z', '_',
];

fn advance(table: &[char], ident: &mut String, i: isize) {
    if ident.len() == 0 {
        if i > 0 {
            ident.push(*table.first().unwrap());
            advance(table, ident, i - 1)
        }
    } else if i != 0 {
        let a = get_last_char(&ident);
        let x = table.iter().copied().position(|y| a == y).unwrap();

        let y = i + x as isize;

        if i < 0 {
            if y < 0 {
                ident.pop();
                advance(table, ident, i + 1)
            } else {
                let t = table[y as usize];
                replace_last_char(ident, t);
            }
        } else if y >= table.len() as isize {
            let mut t = *table.last().unwrap();
            replace_last_char(ident, t);
            t = *table.first().unwrap();
            ident.push(t);
            let tlen = table.len();
            advance(table, ident, y - tlen as isize)
        } else {
            let t = table[y as usize];
            replace_last_char(ident, t);
        }
    }
}

fn get_last_char(x: &str) -> char {
    x.chars().last().unwrap()
}

fn replace_last_char(x: &mut String, c: char) {
    x.pop();
    x.push(c);
}

impl<'a> IdentGen<'a> {
    /// Creates a new instance with the specified table,using the default one if empty.
    #[inline]
    pub const fn new(table: &'a [char]) -> Self {
        Self {
            ident: String::new(),
            table: if table.is_empty() {
                DEFAULT_TABLE
            } else {
                table
            },
        }
    }

    /// Sets a new table,if the new contents not contain the last character then the first one it's
    /// chosen when advancing.
    ///
    /// This will not remove the previous lacking characters.
    ///
    /// # Errors
    ///
    /// Returns a `None` when the table is empty otherwise returns `self`.
    #[inline]
    pub fn set_table(&mut self, table: &'a [char]) -> Option<&mut Self> {
        if table.is_empty() {
            None
        } else {
            self.table = table;
            Some(self)
        }
    }

    /// Convenience for `self.advance(1)`.
    pub fn next(&mut self) -> &str {
        self.advance(1)
    }

    /// Convenience for `self.advance(-1)`.
    pub fn prev(&mut self) -> &str {
        self.advance(-1)
    }

    /// Returns the underlying character table used in [`Self::advance`].
    #[inline]
    pub fn table(&self) -> &[char] {
        self.table
    }

    /// In the case is positive gives you the `i`th possible combination since the one you got,
    /// and in the case of negative gives you the `i`th possible combination prior the one you got,
    ///modifying the underlying `String` and returning a `&str`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ident_gen::IdentGen;
    ///
    /// let mut gen = IdentGen::default();
    ///
    /// assert_eq!(gen.advance(1), "a");
    /// assert_eq!(gen.advance(2), "c");
    /// assert_eq!(gen.advance(-1), "b");
    /// ```
    #[inline]
    pub fn advance(&mut self, i: isize) -> &str {
        advance(self.table, &mut self.ident, i);

        &self.ident
    }

    /// Convenience for `self.ident.clear()`.
    #[inline]
    pub fn clear(&mut self) {
        self.ident.clear()
    }
}

impl<'a> Deref for IdentGen<'a> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.ident
    }
}

impl<'a> DerefMut for IdentGen<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.ident
    }
}

impl<'a> Default for IdentGen<'a> {
    #[inline]
    fn default() -> Self {
        Self::new(&[])
    }
}

impl<'a> Display for IdentGen<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<'a> Debug for IdentGen<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

pub struct IdentGenOwned {
    table: Vec<char>,
    pub ident: String,
}

#[cfg(feature = "serde")]
impl<'a> Serialize for IdentGen<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        IdentGenSerialize { ident: self.ident.clone(), table: self.table.iter().collect() }.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl Serialize for IdentGenOwned {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        IdentGenSerialize { ident: self.ident.clone(), table: self.table.iter().collect() }.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'a> Deserialize<'a> for IdentGenOwned
where Self: 'a
{
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        IdentGenSerialize::deserialize(deserializer).map(|i|Self {
            table: i.table.chars().collect(),
            ident: i.ident            
        })
    }
}

impl IdentGenOwned {
    /// Creates a new instance with the specified table,populating the vec with the default one if empty.
    #[inline]
    pub fn new(mut table: Vec<char>) -> Self {
        if table.is_empty() {
            table.extend(DEFAULT_TABLE);
        }

        Self {
            ident: String::new(),
            table,
        }
    }

    /// Creates a new instance with `x` characters as `table`.
    pub fn from_str_table(x: &str) -> Self {
        Self::new(x.chars().collect())
    }

    /// Convenience for `self.advance(1)`.
    pub fn next(&mut self) -> &str {
        self.advance(1)
    }

    /// Convenience for `self.advance(-1)`.
    pub fn prev(&mut self) -> &str {
        self.advance(-1)
    }

    /// In the case is positive gives you the `i`th possible combination since the one you got,
    /// and in the case of negative gives you the `i`th possible combination prior the one you got,
    ///modifying the underlying `String` and returning a `&str`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ident_gen::IdentGenOwned;
    ///
    /// let mut gen = IdentGenOwned::default();
    ///
    /// assert_eq!(gen.advance(1), "a");
    /// assert_eq!(gen.advance(2), "c");
    /// assert_eq!(gen.advance(-1), "b");
    /// ```
    #[inline]
    pub fn advance(&mut self, i: isize) -> &str {
        advance(&self.table[..], &mut self.ident, i);

        &self.ident
    }

    /// Convenience for `self.ident.clear()`.
    #[inline]
    pub fn clear(&mut self) {
        self.ident.clear()
    }
}

/// Structure used to serialize and deserialize IdentGenOwned.
#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
struct IdentGenSerialize {
    table: String,
    ident: String,
}

impl Deref for IdentGenOwned {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &*self.ident
    }
}

impl DerefMut for IdentGenOwned {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.ident
    }
}

impl Default for IdentGenOwned {
    #[inline]
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Display for IdentGenOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl Debug for IdentGenOwned {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a() {
        let mut ident_gen = IdentGen::new(&['a', 'b', 'c']);

        assert_eq!(ident_gen.next(), "a");
        assert_eq!(ident_gen.next(), "b");
        assert_eq!(ident_gen.next(), "c");
        assert_eq!(ident_gen.next(), "ca");
        assert_eq!(ident_gen.advance(-2), "b");
    }

    #[test]
    fn b() {
        let mut ident_gen = IdentGenOwned::from_str_table("abc");

        assert_eq!(ident_gen.next(), "a");
        assert_eq!(ident_gen.next(), "b");
        assert_eq!(ident_gen.next(), "c");
        assert_eq!(ident_gen.next(), "ca");
        assert_eq!(ident_gen.advance(-2), "b");
    }
}
