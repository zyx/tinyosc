// Copyright (c) 2015 William Light <wrl@illest.net>
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::str;
use std::io::Write;
use std::io;

use byteorder::{ByteOrder, WriteBytesExt, BigEndian};

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Argument<'a> {
    i(i32),
    f(f32),
    d(f64),
    s(&'a str),
    T,
    F,
    None
}

fn strchr(haystack: &[u8], needle: u8) -> Option<usize> {
    for (idx, &hay) in haystack.iter().enumerate() {
        if hay == needle {
            return Some(idx)
        }
    }

    return None
}

impl<'a> Argument<'a> {
    pub fn deserialize(typetag: char, slice: &mut &'a [u8]) -> Result<Argument<'a>, ()> {
        match typetag {
            'T' => Ok(Argument::T),
            'F' => Ok(Argument::F),
            'N' => Ok(Argument::None),

            'i' => {
                let n = BigEndian::read_i32(*slice);
                *slice = &slice[4 ..];
                Ok(Argument::i(n))
            },

            'f' => {
                let n = BigEndian::read_f32(*slice);
                *slice = &slice[4 ..];
                Ok(Argument::f(n))
            },

            'd' => {
                let n = BigEndian::read_f64(*slice);
                *slice = &slice[8 ..];
                Ok(Argument::d(n))
            },


            's' => {
                // find the terminating null
                let next_null = match strchr(slice, 0) {
                    Some(next) => next + 1,
                    None => return Err(())
                };

                let s = match str::from_utf8(&slice[.. (next_null - 1)]) {
                    Ok(s) => s,
                    Err(_) => return Err(())
                };

                // swallow the additional padding
                let pad = match next_null % 4 {
                    0 => 0,
                    pad => 4 - pad,
                };

                *slice = &slice[(next_null + pad) ..];

                Ok(Argument::s(s))
            }

            _ => Err(())
        }
    }

    pub fn typetag(&self) -> char {
        match *self {
            Argument::T => 'T',
            Argument::F => 'F',
            Argument::None => 'N',
            Argument::i(_) => 'i',
            Argument::f(_) => 'f',
            Argument::d(_) => 'd',
            Argument::s(_) => 's'
        }
    }

    pub fn serialize(&self, into: &mut Write) -> io::Result<()> {
        match *self {
            Argument::T | Argument::F | Argument::None => Ok(()),

            Argument::i(arg) => into.write_i32::<BigEndian>(arg),
            Argument::f(arg) => into.write_f32::<BigEndian>(arg),
            Argument::d(arg) => into.write_f64::<BigEndian>(arg),

            Argument::s(arg) => {
                try!(into.write_all(arg.as_ref()));

                let pad = 1 + match (arg.len() + 1) % 4 {
                    0 => 0,
                    pad => 4 - pad
                };

                try!(into.write_all(&[0; 5][.. pad]));

                Ok(())
            }
        }
    }
}

impl<'a> From<bool> for Argument<'a> {
    fn from(b: bool) -> Argument<'a> {
        if b {
            Argument::T
        } else {
            Argument::F
        }
    }
}

impl<'a> From<i32> for Argument<'a> {
    fn from(i: i32) -> Argument<'a> {
        Argument::i(i)
    }
}

impl<'a> From<f32> for Argument<'a> {
    fn from(f: f32) -> Argument<'a> {
        Argument::f(f)
    }
}

impl<'a> From<f64> for Argument<'a> {
    fn from(d: f64) -> Argument<'a> {
        Argument::d(d)
    }
}


impl<'a> From<&'a str> for Argument<'a> {
    fn from(s: &'a str) -> Argument<'a> {
        Argument::s(s)
    }
}
