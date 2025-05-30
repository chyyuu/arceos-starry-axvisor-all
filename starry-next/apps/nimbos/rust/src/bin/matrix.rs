#![no_std]
#![no_main]
#![allow(clippy::needless_range_loop)]

#[macro_use]
extern crate user_lib;

use user_lib::{exit, fork, get_time_us, getpid, sched_yield, wait};

static NUM: usize = 30;
const N: usize = 10;
static P: i32 = 10007;
type Arr = [[i32; N]; N];

fn work(times: isize) {
    let mut a: Arr = Default::default();
    let mut b: Arr = Default::default();
    let mut c: Arr = Default::default();
    for i in 0..N {
        for j in 0..N {
            a[i][j] = 1;
            b[i][j] = 1;
        }
    }
    sched_yield();
    println!("pid {} is running ({} times)!.", getpid(), times);
    for _ in 0..times {
        for i in 0..N {
            for j in 0..N {
                c[i][j] = 0;
                #[allow(clippy::needless_range_loop)]
                for k in 0..N {
                    c[i][j] = (c[i][j] + a[i][k] * b[k][j]) % P;
                }
            }
        }
        for i in 0..N {
            for j in 0..N {
                a[i][j] = c[i][j];
                b[i][j] = c[i][j];
            }
        }
    }
    println!("pid {} done!.", getpid());
    exit(0);
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    for _ in 0..NUM {
        let pid = fork();
        if pid == 0 {
            let current_time = get_time_us();
            let times = (current_time as i32 as isize) * (current_time as i32 as isize) % 1000;
            work(times * 10);
        }
    }

    println!("fork ok.");

    for _ in 0..NUM {
        if wait(None) < 0 {
            panic!("wait failed.");
        }
    }
    assert!(wait(None) < 0);
    println!("matrix passed!");
    0
}
