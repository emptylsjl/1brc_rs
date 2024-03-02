#![feature(slice_split_at_unchecked)]

#[allow(unused)]

use std::collections::HashMap;
use std::{fs, thread};
use std::hint::black_box;
// use std::intrinsics::black_box;
use std::ptr::slice_from_raw_parts;
use std::time::Instant;
use itertools::Itertools;
use rayon::prelude::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use rustc_hash::FxHashMap;


#[derive(Debug, Clone)]
struct Record {
    count: u32,
    sum: f32,
    min: f32,
    max: f32,
}

impl Record {
    fn new(sum: f32, min: f32, max: f32) -> Self {
        Self { count: 1, sum, min, max }
    }
}

// #[derive(Debug, Clone)]
// struct NamedRecord<T> {
//     count: u32,
//     sum: i64,
//     min: f32,
//     max: f32,
//     name: T,
// }
//
// impl<T> NamedRecord<T> {
//     fn new(sum: i64, min: f32, max: f32, name: T) -> Self {
//         Self {  count: 1, sum, min, max, name }
//     }
// }

#[derive(Debug, Clone)]
struct NamedRecord<T, P> {
    count: u32,
    sum: P,
    min: P,
    max: P,
    name: T,
}

impl<T, P> NamedRecord<T, P> {
    fn new(sum: P, min: P, max: P, name: T) -> Self {
        Self {  count: 1, sum, min, max, name }
    }
}

#[derive(Debug, Clone, Default)]
struct HashedRecord<T> {
    count: u32,
    hash: u32,
    sum: f32,
    min: f32,
    max: f32,
    name: T,
}

impl<T> HashedRecord<T> {
    fn new(hash: u32, sum: f32, min: f32, max: f32, name: T) -> Self {
        Self {  count: 1, hash, sum, min, max, name }
    }
}


/// read + raw split_at + noHash + hash + from_utf8 + custom i32parse + thread::spawn + sort in thread + safe get
pub fn impl10(a: &[u8], thread_count: usize) -> String {
    let chunk_size = a.len() / thread_count + 1;
    let mut pos = 0;
    let mut tks = (1..thread_count).map(|x| {
        let st = pos;
        pos = 1 + x * chunk_size + (&a[x * chunk_size..]).iter().position(|i| i == &10).unwrap();
        &a[st..pos]
    }).collect::<Vec<_>>();
    tks.push(&a[pos..]);

    use nohash::BuildNoHashHasher;
    let mut out_handle = tks.iter().map(|chunk| {
        let ck = unsafe { &*slice_from_raw_parts(chunk.as_ptr(), chunk.len()) };
        thread::spawn(move || {
            // let mut out = HashMap::<u32, NamedRecord<&[u8], f32>, BuildNoHashHasher<u8>>::with_capacity_and_hasher(1000, BuildNoHashHasher::default());
            let mut out = FxHashMap::<u32, NamedRecord<&[u8], i32>>::with_capacity_and_hasher(1000, Default::default());
            for raw in ck[..ck.len() - 1].split(|x| x == &b'\n') {
                let index = raw.iter().position(|x| x == &b';').unwrap();
                let (name, value) = unsafe { raw.split_at_unchecked(index) };
                let value = {
                    let b = unsafe { &*(value as *const _ as *const [i8]) };
                    let g = |i: usize| { unsafe { *b.get_unchecked(i) } };
                    let sign = g(1) == 45;
                    let sign_u = sign as usize;
                    let dot =
                        if g(2) == 46 { 1 }
                        else if g(3) == 46 { 2 }
                        else { 3 } ;
                    let mut val = (g(sign_u+1) - 48);
                    if dot - sign_u == 2 {
                        val = val * 10 + (g(sign_u+2) - 48)
                    }
                    ((val as i32) * 10 + (g(dot+2) - 48) as i32) * (((!sign as i32) << 1) - 1)
                };

                // let hash = name.iter().fold(0u32, |a, b| (a << 5) + a + (*b as u32));
                let mut hash = 0;
                (0..9).for_each(|x| {
                    hash = (hash << 5) + hash + (*name.get(x).unwrap_or(&0) as u32);
                });
                if let Some(p) = out.get_mut(&hash) {
                    p.sum += value;
                    p.min = value.min(p.min);
                    p.max = value.max(p.max);
                    p.count += 1;
                } else {
                    out.insert(hash, NamedRecord::new(value, value, value, name));
                }
            }
            let mut out = out.into_iter().collect::<Vec<_>>();
            out.sort_by_key(|x| std::str::from_utf8(x.1.name).unwrap());
            out
        })
    }).collect::<Vec<_>>();

    let first = out_handle.pop().unwrap().join().unwrap();
    let out = out_handle.into_iter().fold(first, |mut a, b| {
        let b = b.join().unwrap();
        a.iter_mut().zip(b).for_each(|(x, (k, v))| {
            x.1.sum += v.sum;
            x.1.min = v.min.min(x.1.min);
            x.1.max = v.max.max(x.1.max);
            x.1.count += v.count;
        });
        a
    });

    let out = out.iter().map(|x| {
        format!(
            "{}={:.1}/{:.1}/{:.1}",
            std::str::from_utf8(x.1.name).unwrap(),
            x.1.min as f32 * 0.1,
            (x.1.sum as f32 / x.1.count as f32) * 0.1,
            x.1.max as f32 * 0.1
        )
    }).join(", ");
    let out = format!("{{{out}}}");
    out
}

/// read + fxHashMap + hash + custom f32parse + thread::spawn + sort in thread + safe get
pub fn impl09(a: &[u8], thread_count: usize) -> String {
    let chunk_size = a.len() / thread_count + 1;
    let mut pos = 0;
    let mut tks = (1..thread_count).map(|x| {
        let st = pos;
        pos = 1 + x * chunk_size + (&a[x * chunk_size..]).iter().position(|i| i == &10).unwrap();
        &a[st..pos]
    }).collect::<Vec<_>>();
    tks.push(&a[pos..]);

    let mut out_handle = tks.iter().map(|chunk| {
        let ck = unsafe { &*slice_from_raw_parts(chunk.as_ptr(), chunk.len()) };
        thread::spawn(move || {
            let m = |x: bool| if x { 0b1111 } else { 0 };
            let c = |x: bool| x as i8;
            let mut out = FxHashMap::<u32, NamedRecord<&[u8], f32>>::with_capacity_and_hasher(1000, Default::default());
            for raw in ck[..ck.len() - 1].split(|x| x == &b'\n') {
                let index = raw.iter().position(|x| x == &b';').unwrap();
                let (name, value) = unsafe {
                    raw.split_at_unchecked(index)
                    // let ptr = raw.as_ptr();
                    // unsafe { (from_raw_parts(ptr, index), from_raw_parts(ptr.add(index), raw.len() - index)) }
                };
                let value = {
                    let b = unsafe { &*(value as *const _ as *const [i8]) };
                    let len = b.len();

                    // let b1 = unsafe { *b.get_unchecked(1) };
                    // let b2 = unsafe { *b.get_unchecked(2) };
                    // let b3 = unsafe { *b.get_unchecked(3) };
                    // let b4 = unsafe { *b.get_unchecked(4) };
                    // let b5 = unsafe { *b.get_unchecked(5) };

                    let b1 = *b.get(1).unwrap_or(&0);
                    let b2 = *b.get(2).unwrap_or(&0);
                    let b3 = *b.get(3).unwrap_or(&0);
                    let b4 = *b.get(4).unwrap_or(&0);
                    let b5 = *b.get(5).unwrap_or(&0);

                    let sign = if b1 != 45 { 1.0 } else { -1.0 };
                    let o1 = m(b1 != 45) & (b1 - 48);
                    let o2 = (m(b2 != 46) & (b2 - 48)) + (o1 * (10 >> (3 & m(b2 == 46))));
                    let o3 = ((m(b3 != 46) & (b3 - 48)) as f32 * if b2 == 46 { 0.1 } else { 1.0 }) + (o2 * (10 >> (3 & m(b4 != 46)))) as f32;
                    let o4 = (m(b4 != 46) & m(len > 4) & (b4 - 48)) as f32 * 0.1 + o3;
                    let o5 = (m(len > 5) & (b5 - 48)) as f32 * 0.1 + o4;
                    o5 * sign
                };

                // let hash = name.iter().fold(0u32, |a, b| (a << 5) + a + (*b as u32));
                let mut hash = 0;
                (0..9).for_each(|x| {
                    hash = (hash << 5) + hash + (*name.get(x).unwrap_or(&0) as u32);
                });
                if let Some(p) = out.get_mut(&hash) {
                    p.sum += value;
                    p.min = value.min(p.min);
                    p.max = value.max(p.max);
                    p.count += 1;
                } else {
                    out.insert(hash, NamedRecord::new(value, value, value, name));
                }
            }
            let mut out = out.into_iter().collect::<Vec<_>>();
            out.sort_by_key(|x| std::str::from_utf8(x.1.name).unwrap());
            out
        })
    }).collect::<Vec<_>>();

    let first = out_handle.pop().unwrap().join().unwrap();
    let out = out_handle.into_iter().fold(first, |mut a, b| {
        let b = b.join().unwrap();
        a.iter_mut().zip(b).for_each(|(x, (k, v))| {
            x.1.sum += v.sum;
            x.1.min = v.min.min(x.1.min);
            x.1.max = v.max.max(x.1.max);
            x.1.count += v.count;
        });
        a

        // b.into_iter().for_each(|(k, v)| {
        //     let item = a.get_mut(&k).unwrap();
        //     item.sum += v.sum;
        //     item.min = v.min.min(item.min);
        //     item.max = v.max.max(item.max);
        //     item.count += v.count;
        // });
        // a
    });

    // let mut out = out.into_iter().collec::<Vec<_>>t();
    // out.sort_by_key(|x| std::str::from_utf8(x.1.name).unwrap());

    // let a = out.iter().map(|x| x.1.sum).collect_vec();
    // println!("{:?}", a.iter().fold(0f32, |a, &b| a.max(b)));
    // println!("{:?}", a.iter().fold(0f32, |a, &b| a.min(b)));

    let out = out.iter().map(|x| {
        format!("{}={}/{:.1}/{}", std::str::from_utf8(x.1.name).unwrap(), x.1.min, x.1.sum / x.1.count as f32, x.1.max)
    }).join(", ");
    let out = format!("{{{out}}}");
    out
}

/// read + hashmap + hash + custom f32parse + get_unchecked + thread::spawn + sort in thread
pub fn impl08(a: &[u8], thread_count: usize) -> String {
    let chunk_size = a.len() / thread_count + 1;
    let mut pos = 0;
    let mut tks = (1..thread_count).map(|x| {
        let st = pos;
        pos = 1 + x * chunk_size + (&a[x * chunk_size..]).iter().position(|i| i == &10).unwrap();
        &a[st..pos]
    }).collect::<Vec<_>>();
    tks.push(&a[pos..]);

    let mut out_handle = tks.iter().map(|chunk| {
        let ck = unsafe { &*slice_from_raw_parts(chunk.as_ptr(), chunk.len()) };
        thread::spawn(move || {
            let mut out = HashMap::<u32, NamedRecord<&[u8], f32>>::with_capacity(1000);
            for raw in ck[..ck.len() - 1].split(|x| x == &b'\n') {
                let index = raw.iter().position(|x| x == &b';').unwrap();
                let (name, value) = unsafe { raw.split_at_unchecked(index) };
                let value = {
                    let m = |x: bool| if x { 0b1111 } else { 0 };
                    let c = |x: bool| x as i8;
                    let b = unsafe { &*(value as *const _ as *const [i8]) };
                    let len = black_box(b.len());

                    let b1 = unsafe { *b.get_unchecked(1) };
                    let b2 = unsafe { *b.get_unchecked(2) };
                    let b3 = unsafe { *b.get_unchecked(3) };
                    let b4 = unsafe { *b.get_unchecked(4) };
                    let b5 = unsafe { *b.get_unchecked(5) };

                    let sign = if b1 != 45 { 1.0 } else { -1.0 };
                    let o1 = m(b1 != 45) & (b1 - 48);
                    let o2 = (m(b2 != 46) & (b2 - 48)) + (o1 * (10 >> (3 & m(b2 == 46))));
                    let o3 = ((m(b3 != 46) & (b3 - 48)) as f32 * if b2 == 46 { 0.1 } else { 1.0 }) + (o2 * (10 >> (3 & m(b4 != 46)))) as f32;
                    let o4 = (m(b4 != 46) & m(len > 4) & (b4 - 48)) as f32 * 0.1 + o3;
                    let o5 = (m(len > 5) & (b5 - 48)) as f32 * 0.1 + o4;
                    o5 * sign
                };

                let hash = name.iter().fold(0u32, |a, b| (a << 5) + a + (*b as u32));
                if let Some(p) = out.get_mut(&hash) {
                    p.sum += value;
                    p.min = value.min(p.min);
                    p.max = value.max(p.max);
                    p.count += 1;
                } else {
                    out.insert(hash, NamedRecord::new(value, value, value, name));
                }
            }
            let mut out = out.into_iter().collect::<Vec<_>>();
            out.sort_by_key(|x| std::str::from_utf8(x.1.name).unwrap());
            out
        })
    }).collect::<Vec<_>>();

    let first = out_handle.pop().unwrap().join().unwrap();
    let out = out_handle.into_iter().fold(first, |mut a, b| {
        let b = b.join().unwrap();
        a.iter_mut().zip(b).for_each(|(x, (k, v))| {
            x.1.sum += v.sum;
            x.1.min = v.min.min(x.1.min);
            x.1.max = v.max.max(x.1.max);
            x.1.count += v.count;
        });
        a
    });

    let out = out.iter().map(|x| {
        format!("{}={}/{:.1}/{}", std::str::from_utf8(x.1.name).unwrap(), x.1.min, x.1.sum / x.1.count as f32, x.1.max)
    }).join(", ");
    let out = format!("{{{out}}}");
    out
}

/// read + hashmap + hash + custom f32parse + get_unchecked + par_iter
pub fn impl07(a: &[u8], thread_count: usize) -> String {
    let chunk_size = a.len() / thread_count + 1;
    let mut pos = 0;
    let mut tks = (1..thread_count).map(|x| {
        let st = pos;
        pos = 1 + x * chunk_size + (&a[x * chunk_size..]).iter().position(|i| i == &10).unwrap();
        &a[st..pos]
    }).collect::<Vec<_>>();
    tks.push(&a[pos..]);

    let out = tks.par_iter().map(|chunk| {
        let mut out = HashMap::<u32, NamedRecord<&[u8], f32>>::with_capacity(1000);
        for raw in chunk[..chunk.len() - 1].split(|x| x == &b'\n') {
            let index = raw.iter().position(|x| x == &b';').unwrap();
            let (name, value) = unsafe { raw.split_at_unchecked(index) };

            let value = {
                let m = |x: bool| if x { 0b1111 } else { 0 };
                let c = |x: bool| x as i8;
                let b = unsafe { &*(value as *const _ as *const [i8]) };
                let len = black_box(b.len());

                let b1 = unsafe { *b.get_unchecked(1) };
                let b2 = unsafe { *b.get_unchecked(2) };
                let b3 = unsafe { *b.get_unchecked(3) };
                let b4 = unsafe { *b.get_unchecked(4) };
                let b5 = unsafe { *b.get_unchecked(5) };

                let sign = if b1 != 45 { 1.0 } else { -1.0 };
                let o1 = m(b1 != 45) & (b1 - 48);
                let o2 = (m(b2 != 46) & (b2 - 48)) + (o1 * (10 >> (3 & m(b2 == 46))));
                let o3 = ((m(b3 != 46) & (b3 - 48)) as f32 * if b2 == 46 { 0.1 } else { 1.0 }) + (o2 * (10 >> (3 & m(b4 != 46)))) as f32;
                let o4 = (m(b4 != 46) & m(len > 4) & (b4 - 48)) as f32 * 0.1 + o3;
                let o5 = (m(len > 5) & (b5 - 48)) as f32 * 0.1 + o4;
                o5 * sign
            };

            let hash = name.iter().fold(0u32, |a, b| (a << 5) + a + (*b as u32));
            if let Some(p) = out.get_mut(&hash) {
                p.sum += value;
                p.min = value.min(p.min);
                p.max = value.max(p.max);
                p.count += 1;
            } else {
                out.insert(hash, NamedRecord::new(value, value, value, name));
            }
        }
        out
    }).collect::<Vec<_>>().into_iter().fold(HashMap::<u32, NamedRecord<&[u8], f32>>::with_capacity(1000), |mut a, b| {
        b.into_iter().for_each(|(k, v)| {
            if let Some(item) = a.get_mut(&k) {
                item.sum += v.sum;
                item.min = v.min.min(item.min);
                item.max = v.max.max(item.max);
                item.count += v.count;
            } else {
                a.insert(k, v);
            }
        });
        a
    });


    let mut out = out.into_iter().collect::<Vec<_>>();
    out.sort_by_key(|x| std::str::from_utf8(x.1.name).unwrap());

    let out = out.iter().map(|x| {
        format!("{}={}/{:.1}/{}", std::str::from_utf8(x.1.name).unwrap(), x.1.min, x.1.sum / x.1.count as f32, x.1.max)
    }).join(", ");
    let out = format!("{{{out}}}");
    out
}

/// read + hashmap + hash + custom f32parse + get_unchecked
pub fn impl06(a: &[u8]) -> String {
    let mut out = HashMap::<u32, NamedRecord<&[u8], f32>>::with_capacity(1000);
    for raw in a[..a.len() - 1].split(|x| x == &b'\n') {
        let index = raw.iter().position(|x| x == &b';').unwrap();
        let (name, value) = raw.split_at(index);

        let value = {
            let m = |x: bool| if x { 0b1111 } else { 0 };
            let c = |x: bool| x as i8;
            let b = unsafe { &*(value as *const _ as *const [i8]) };
            let len = black_box(b.len());

            let b1 = unsafe { *b.get_unchecked(1) };
            let b2 = unsafe { *b.get_unchecked(2) };
            let b3 = unsafe { *b.get_unchecked(3) };
            let b4 = unsafe { *b.get_unchecked(4) };
            let b5 = unsafe { *b.get_unchecked(5) };

            let sign = if b1 != 45 { 1.0 } else { -1.0 };
            let o1 = m(b1 != 45) & (b1 - 48);
            let o2 = (m(b2 != 46) & (b2 - 48)) + (o1 * (10 >> (3 & m(b2 == 46))));
            let o3 = ((m(b3 != 46) & (b3 - 48)) as f32 * if b2 == 46 { 0.1 } else { 1.0 }) + (o2 * (10 >> (3 & m(b4 != 46)))) as f32;
            let o4 = (m(b4 != 46) & m(len > 4) & (b4 - 48)) as f32 * 0.1 + o3;
            let o5 = (m(len > 5) & (b5 - 48)) as f32 * 0.1 + o4;
            o5 * sign
        };

        let hash = name.iter().fold(0u32, |a, b| (a << 5) + a + (*b as u32));
        if let Some(p) = out.get_mut(&hash) {
            p.sum += value;
            p.min = value.min(p.min);
            p.max = value.max(p.max);
            p.count += 1;
        } else {
            out.insert(hash, NamedRecord::new(value, value, value, name));
        }
    }
    let mut out = out.into_iter().collect::<Vec<_>>();
    out.sort_by_key(|x| std::str::from_utf8(x.1.name).unwrap());
    let out = out.iter().map(|x| {
        format!("{}={}/{:.1}/{}", std::str::from_utf8(x.1.name).unwrap(), x.1.min, x.1.sum / x.1.count as f32, x.1.max)
    }).join(", ");
    let out = format!("{{{out}}}");
    out
}

/// read + unsafe split_at + hashmap + hash + custom f32parse
pub fn impl05(a: &[u8]) -> String {
    let mut out = HashMap::<u32, NamedRecord<&[u8], f32>>::with_capacity(1000);
    for raw in a[..a.len() - 1].split(|x| x == &b'\n') {
        let index = raw.iter().position(|x| x == &b';').unwrap();
        let (name, value) = raw.split_at(index);

        let value = {
            let m = |x: bool| if x { 0b1111 } else { 0 };
            let c = |x: bool| x as i8;
            let b = unsafe { &*(value as *const _ as *const [i8]) };
            let len = b.len();

            let b1 = *b.get(1).unwrap();
            let b2 = *b.get(2).unwrap();
            let b3 = *b.get(3).unwrap();
            let b4 = *b.get(4).unwrap_or(&0);
            let b5 = *b.get(5).unwrap_or(&0);

            let sign = if b1 != 45 { 1.0 } else { -1.0 };
            let o1 = m(b1 != 45) & (b1 - 48);
            let o2 = (m(b2 != 46) & (b2 - 48)) + (o1 * (10 >> (3 & m(b2 == 46))));
            let o3 = ((m(b3 != 46) & (b3 - 48)) as f32 * if b2 == 46 { 0.1 } else { 1.0 }) + (o2 * (10 >> (3 & m(b4 != 46)))) as f32;
            let o4 = (m(b4 != 46) & m(len > 4) & (b4 - 48)) as f32 * 0.1 + o3;
            let o5 = (m(len > 5) & (b5 - 48)) as f32 * 0.1 + o4;
            o5 * sign
        };

        let hash = name.iter().fold(0u32, |a, b| (a << 5) + a + (*b as u32));
        if let Some(p) = out.get_mut(&hash) {
            p.sum += value;
            p.min = value.min(p.min);
            p.max = value.max(p.max);
            p.count += 1;
        } else {
            out.insert(hash, NamedRecord::new(value, value, value, name));
        }
    }
    let mut out = out.into_iter().collect::<Vec<_>>();
    out.sort_by_key(|x| std::str::from_utf8(x.1.name).unwrap());
    let out = out.iter().map(|x| {
        format!("{}={}/{:.1}/{}", std::str::from_utf8(x.1.name).unwrap(), x.1.min, x.1.sum / x.1.count as f32, x.1.max)
    }).join(", ");
    let out = format!("{{{out}}}");
    out
}

/// read_str + hashmap + hash
pub fn impl04(a: &str) -> String {
    let mut out = HashMap::<u32, NamedRecord<&[u8], f32>>::with_capacity(1000);
    for i in a[..a.len() - 1].split("\n") {
        let raw = i.as_bytes();
        let index = raw.iter().position(|x| x == &b';').unwrap();
        let (name, value) = raw.split_at(index);
        let value = std::str::from_utf8(&value[1..]).unwrap().parse::<f32>().unwrap();
        let hash = name.iter().fold(0u32, |a, b| (a << 5) + a + (*b as u32));
        if let Some(p) = out.get_mut(&hash) {
            p.sum += value;
            p.min = value.min(p.min);
            p.max = value.max(p.max);
            p.count += 1;
        } else {
            out.insert(hash, NamedRecord::new(value, value, value, name));
        }
    }
    let mut out = out.into_iter().collect::<Vec<_>>();
    out.sort_by_key(|x| std::str::from_utf8(x.1.name).unwrap());
    let out = out.iter().map(|x| {
        format!("{}={}/{:.1}/{}", std::str::from_utf8(x.1.name).unwrap(), x.1.min, x.1.sum / x.1.count as f32, x.1.max)
    }).join(", ");
    let out = format!("{{{out}}}");
    out
}

/// read_str + pure str impl + hashmap
pub fn impl03(a: &str) -> String {
    let mut out = HashMap::<&str, Record>::with_capacity(1000);
    for i in a[..a.len() - 1].split("\n") {
        let mut sp = i.splitn(2, ';');
        let (name, value) = (sp.next().unwrap(), sp.next().unwrap().parse::<f32>().unwrap());
        if let Some(p) = out.get_mut(name) {
            p.sum += value;
            p.min = value.min(p.min);
            p.max = value.max(p.max);
            p.count += 1;
        } else {
            out.insert(name, Record::new(value, value, value));
        }
    }
    let mut out = out.into_iter().collect::<Vec<_>>();
    out.sort_by_key(|x| x.0);
    let out = out.iter().map(|x| {
        format!("{}={}/{:.1}/{}", x.0, x.1.min, x.1.sum / x.1.count as f32, x.1.max)
    }).join(", ");
    let out = format!("{{{out}}}");
    out
}

/// read_str + string split + vec + hash
pub fn impl02(a: &str) -> String {
    let mut out = Vec::<HashedRecord<&str>>::with_capacity(1000);
    for i in a[..a.len() - 1].split("\n") {
        let mut sp = i.splitn(2, ';');
        let (name, value) = (sp.next().unwrap(), sp.next().unwrap().parse::<f32>().unwrap());
        let hash = name.as_bytes().iter().fold(0u32, |a, b| (a << 5) + a + (*b as u32));
        let pos = out.iter().position(|x| x.hash == hash);
        if let Some(p) = pos {
            out[p].sum += value;
            out[p].min = value.min(out[p].min);
            out[p].max = value.max(out[p].max);
            out[p].count += 1;
        } else {
            out.push(HashedRecord::new(hash, value, value, value, name))
        }
    }
    out.sort_by_key(|x| x.name);
    let out = out.iter().map(|x| {
        format!("{}={}/{:.1}/{}", x.name, x.min, x.sum / x.count as f32, x.max)
    }).join(", ");
    let out = format!("{{{out}}}");
    out
}

/// read_str + vec + hash + from_utf8
pub fn impl01(a: &str) -> String {
    let mut out = Vec::<HashedRecord<&[u8]>>::with_capacity(1000);
    for i in a[..a.len() - 1].split("\n") {
        let raw = i.as_bytes();
        let index = raw.iter().position(|x| x == &b';').unwrap();
        let (name, value) = raw.split_at(index);
        let value = std::str::from_utf8(&value[1..]).unwrap().parse::<f32>().unwrap();
        let hash = name.iter().fold(0u32, |a, b| (a << 5) + a + (*b as u32));
        let pos = out.iter().position(|x| x.hash == hash);
        if let Some(p) = pos {
            out[p].sum += value;
            out[p].min = value.min(out[p].min);
            out[p].max = value.max(out[p].max);
            out[p].count += 1;
        } else {
            out.push(HashedRecord::new(hash, value, value, value, name))
        }
    }
    out.sort_by_key(|x| std::str::from_utf8(x.name).unwrap());
    let out = out.iter().map(|x| {
        format!("{}={}/{:.1}/{}", std::str::from_utf8(x.name).unwrap(), x.min, x.sum / x.count as f32, x.max)
    }).join(", ");
    let out = format!("{{{out}}}");
    out
}