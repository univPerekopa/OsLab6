use std::sync::atomic::Ordering;
use std::sync::{atomic::{AtomicU32}, Arc, Mutex, Barrier};
use std::time::Instant;

fn _2a() {
    let a = Arc::new(Mutex::new(0u32));
    let b = a.clone();
    let c = a.clone();

    let h1 = std::thread::spawn(move || {
        for _ in 0..10_000_000 {
            *a.lock().unwrap() += 1;
        }
    });
    let h2 = std::thread::spawn(move || {
        for _ in 0..10_000_000 {
            *b.lock().unwrap() += 1;
        }
    });
    h1.join().unwrap();
    h2.join().unwrap();
    println!("Result x={}", *c.lock().unwrap());
}

fn _2_3() {
    let a = Arc::new(AtomicU32::new(0));
    let b = a.clone();
    let c = a.clone();

    let h1 = std::thread::spawn(move || {
        for _ in 0..10_000_000 {
            b.fetch_add(1, Ordering::Relaxed);
        }
    });
    let h2 = std::thread::spawn(move || {
        for _ in 0..10_000_000 {
            c.fetch_add(1, Ordering::Relaxed);
        }
    });
    h1.join().unwrap();
    h2.join().unwrap();
    println!("Result x={:?}", a.load(Ordering::Relaxed));
}

#[derive(Debug, Clone, Copy)]
struct RefU32(pub *mut u32);
unsafe impl Send for RefU32 {}
unsafe impl Sync for RefU32 {}

unsafe fn _2b() {
    let mut x = 0u32;
    let y = RefU32(&mut x);
    let h1 = std::thread::spawn(move || {
        let z = y;
        for _ in 0..10_000_000 {
            *z.0 += 1;
        }
    });
    let h2 = std::thread::spawn(move || {
        let z = y;
        for _ in 0..10_000_000 {
            *z.0 += 1;
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
    println!("Result x={}", x);
}

unsafe fn _2_3_star() {
    let mut x = 0u32;
    let y = RefU32(&mut x);
    let mut barriers1 = Vec::new();
    barriers1.resize(1000, Arc::new(Barrier::new(2)));
    let barriers2 = barriers1.clone();
    let h1 = std::thread::spawn(move || {
        let z = y;
        for i in 0..1000 {
            let t = *z.0;
            barriers1[i].wait();
            *z.0 = t + 1;
        }
    });
    let h2 = std::thread::spawn(move || {
        let z = y;
        for i in 0..1000 {
            let t = *z.0;
            barriers2[i].wait();
            *z.0 = t + 1;
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
    println!("Result x={}", x);
}

fn main() {
    let mut started_at: Instant = Instant::now();
    _2a();
    println!("2a took {:?}", started_at.elapsed());

    started_at = Instant::now();
    unsafe {
        _2b();
    }
    println!("2b took {:?}", started_at.elapsed());

    started_at = Instant::now();
    _2_3();
    println!("2.3 took {:?}", started_at.elapsed());

    started_at = Instant::now();
    unsafe {
        _2_3_star();
    }
    println!("2.3* took {:?}", started_at.elapsed());
}
