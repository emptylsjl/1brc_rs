use std::{fs, time};
use std::collections::HashSet;
use std::hint::black_box;
use std::process::exit;
use itertools::Itertools;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use rustc_hash::FxHashSet;
use similar::{ChangeTag, TextDiff};
use rand::{random, Rng};
use rs_1brc::*;

fn main() {

    let now = time::Instant::now();
    let f_name = "measure_1b.txt";
    let raw = fs::read(f_name).unwrap();
    println!("{:?}", now.elapsed());

    let now = time::Instant::now();
    let a = impl10(&raw, 22);

    println!("{}", a);
    println!("{:?}", now.elapsed());

    // test
    #[cfg(any)]
    {
        let a = [
            ";22.0",
            ";22.9",
            ";-22.0",
            ";-22.9",
            ";1.9",
            ";1.0",
            ";0.6",
            ";-1.9",
            ";-0.0",
            ";-0.6",
        ];


        for s in a {
            let m = |x: bool| if x { 0b1111 } else { 0 };
            let c = |x: bool| x as i8;

            let b = unsafe { &*(s.as_bytes() as *const _ as *const [i8]) };
            let len = black_box(b.len());
            // let len = (b.len());

            let b1 = *b.get(1).unwrap();
            let b2 = *b.get(2).unwrap();
            let b3 = *b.get(3).unwrap();
            let b4 = *b.get(4).unwrap_or(&0);
            let b5 = *b.get(5).unwrap_or(&0);

            // let b1 = unsafe { *b.get_unchecked(1) };
            // let b2 = unsafe { *b.get_unchecked(2) };
            // let b3 = unsafe { *b.get_unchecked(3) };
            // let b4 = unsafe { *b.get_unchecked(4) };
            // let b5 = unsafe { *b.get_unchecked(5) };

            print!("{:8} |", s);

            // // let sign = (((b1!=45) as i8) << 1) - 1;
            // let sign = if b1!=45 { 1.0 } else { -1.0 };
            //
            // let o1 = m(b1!=45) & (b1-48);
            // // let o1 = if b1!=45 { b1-48 } else { 0 };
            //
            // // let mut dot = false;
            //
            // let o2 = (m(b2!=46) & (b2-48)) + (o1 * (10 >> (3 & m(b2==46))));
            // // let o2 = (m(b2!=46) & (b2-48)) + (o1 * (if b2==46 { 1 } else { 10 }));
            //
            // let o3 = ((m(b3!=46) & (b3-48)) as f32 * if b2==46 {0.1} else {1.0}) + (o2 * (10 >> (3 & m(b4!=46)))) as f32;
            // // let o3 = if b2==46 { (m(b3!=46) & (b3-48)) as f32 * 0.1 } else {(m(b3!=46) & (b3-48)) as f32 } + (o2 * (10 >> (3 & m(b4!=46)))) as f32;
            //
            // let o4 = (m(b4!=46) & m(len>4) & (b4-48)) as f32 * 0.1 + o3;
            //
            // let o5 = (m(len>5) & (b5-48)) as f32 * 0.1 + o4;

            // impl #2
            let g = |i: usize| { unsafe { *b.get_unchecked(i) } };
            let sign = g(1) == 45;
            let sign_u = sign as usize;
            let dot =
                if g(2) == 46 { 1 } else if g(3) == 46 { 2 } else { 3 };
            let mut val = (g(sign_u + 1) - 48);
            if dot - sign_u == 2 {
                val = val * 10 + (g(sign_u + 2) - 48)
            }
            let val = ((val as i32) * 10 + (g(dot + 2) - 48) as i32) * (((sign as i32) << 1) - 1);

            // // print!("{:5}", b4 as u8 as char);
            // // print!("{:5}", (10 >> (3 & m(b4!=46))));
            // // print!("{:5}", (10 >> (3 & m(b4!=46))) * o2);
            print!("{:5.1}", val);
            // print!("{:5.1}", len>5);
            // print!("{:5.1}  ", (m(b4!=46) & m(len>5) & (b4-48)) as f32 * 0.1);

            println!();

            // println!("{} > 5: {} {}", b.len(), b.len()>5, black_box(b.len())>5);
        }
    }
    // println!("0b{:b}", b & 0b00000);
    // println!("0b{:b}", 3 & 0b1111);
}
