#![no_std]

extern crate alloc;

use alloc::string::String;
use core::ops::{Deref, DerefMut};

/// Identifier generator,it can generate all possible combinations for a table for all possible lengths
/// in order,the default table it's [`DEFAULT_TABLE`].
pub struct IdentGen<'a> {
    pub ident: String,
    table: &'a [char],
}

/// All characters that are in the `snake_case` standard.
pub const DEFAULT_TABLE: &'static [char] = &[
    'a', 'b', 'c', 'd', 'f', 'e', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'z', '_'
];

/// All characters that are in the upper `SNAKE_CASE` standard.
pub const UPPER_SNAKE: &'static [char] = &[
    'A', 'B', 'C', 'D', 'F', 'E', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Z', '_'
];

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

    /// Returns the underlying character table used in [`Self::advance`] and [`Self::regress`].
    #[inline]
    pub fn table(&self) -> &[char] {
        self.table
    }

    /// Convenience to `self.advance(1)`.
    #[inline]
    pub fn next(&mut self) -> &str {
        self.advance(1)
    }

    /// Convenience to `self.regress(1)`.
    #[inline]
    pub fn prev(&mut self) -> &str {
        self.regress(1)
    }

    /// Returns `self.advance(i)` if `i.is_positive()` otherwise `self.regress(i.abs())`.
    #[inline]
    pub fn letter_by(&mut self, i: isize) -> &str {
        if i.is_positive() {
            self.advance(i as usize)
        } else {
            self.regress(i.abs() as usize)
        }
    }

    /// Gives you the `i`th possible combination since the one you got,modifying the underlying
    /// `String` and returning a `&str`.
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
    /// 
    /// gen.clear();
    /// 
    /// assert_eq!(gen.advance(3), "c");
    /// ```
    pub fn advance(&mut self, i: usize) -> &str {
            let Self { ref mut ident, table } = *self;

            if i == 0 {
                return self
            }

            if ident.is_empty()  {
                ident.push(table[0]);
                self.advance(i - 1);
                self
            } else {

                let mut index = 0;
                let mut n = table.iter().copied().position(|e| ident.starts_with(e)).unwrap_or(0) + i;
                let mut next_char = next_char_boundary(&ident, index);
                let mut len = next_char.unwrap_or(ident.len());

                while n >= table.len() {
                    unsafe { replace_char(ident, index, len, *table.last().unwrap()); }
                    let diff = n - table.len() + 1;

                    if let Some(len_previous) = next_char {

                        index = len_previous;

                        next_char = next_char_boundary(&ident, index);
                        len = next_char.unwrap_or(ident.len());

                        let s = &ident[index..];

                        if s == "" {
                            continue
                        }
 
                        n = table.iter().copied().position(|e| s.starts_with(e)).unwrap_or(0) + diff;
                    } else {
                        ident.push(table[0]);
                        self.advance(diff - 1);
                        return self
                    }
                }

                unsafe { replace_char(ident, index, len, table[n]); }
                self
            }
    }

    /// Gives you the `i`th possible combination prior the one you got,modifying the underlying
    /// `String` and returning a `&str`,being empty if you past the first one.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use ident_gen::IdentGen;
    /// 
    /// let mut gen = IdentGen::default();
    /// 
    /// assert_eq!(gen.advance(1), "a");
    /// assert_eq!(gen.regress(1), "");
    /// assert_eq!(gen.advance(2), "b");
    /// assert_eq!(gen.regress(1), "a");
    /// ```
    pub fn regress(&mut self, i: usize) -> &str {
            let Self { ref mut ident, table } = *self;

            if i == 0 {
                return self
            }

            let i = i as isize;

            if ident.is_empty()  {
                self
            } else {
                let mut n = table.iter().copied().position(|e| ident.ends_with(e)).unwrap_or(0) as isize - i;
                let mut prev_char = prev_char_boundary(&ident, ident.len());
                let mut index = prev_char.unwrap_or(0);
                let mut len = ident.len();

                while n < 0 {
                    ident.pop();

                    if let Some(len_previous) = prev_char {

                        index = len_previous;

                        prev_char = prev_char_boundary(&ident, index);
                        len = prev_char.unwrap_or(0);

                        let s = &ident[..len];

                        if s == "" {
                            break
                        }
 
                        n = table.iter().copied().position(|e| s.ends_with(e)).unwrap_or(0) as isize + n;
                    } else {
                        break
                    }
                }

                unsafe { replace_char(ident, index, len, table[n as usize]); }
                self
            }
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

#[inline]
fn next_char_boundary(s: &str, index: usize) -> Option<usize> {
    char_boundary(s, index, true)
}

#[inline]
fn prev_char_boundary(s: &str, index: usize) -> Option<usize> {
    char_boundary(s, index, false)
}

#[inline]
fn char_boundary(s: &str, mut index: usize, forward: bool) -> Option<usize> {
    let f = if forward { usize::checked_add } else { usize::checked_sub };

    loop {
        index = f(index, 1)?;        

        if s.len() < index {
            break None
        }

        if s.is_char_boundary(index) {
            break Some(index)
        } 
    }
}

use core::fmt::{self, Debug, Display, Formatter};

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

fn bytes<T: ?Sized>(x: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts(x as *const T as *const u8, core::mem::size_of_val(x)) }
}

unsafe fn replace_char(s: &mut String, start: usize, end: usize, ch: char) {
    let len_ch = ch.len_utf8();
    let mut len = end - start;    

    if len > len_ch {
        s.as_mut_vec().drain(end - (len - len_ch)..end);
        len = len_ch;
    }

    s.as_mut_vec().splice(start..end, bytes(&ch).iter().copied().take(len));
}
