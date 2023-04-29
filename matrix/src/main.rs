use std::env;
use std::time::Instant;

fn gen_random_row(n: usize) -> Vec<u64> {
    (0..n).map(|_| rand::random::<u64>() % 10000).collect()
}

fn gen_matrices(n: usize, m: usize, k: usize) -> (Vec<Vec<u64>>, Vec<Vec<u64>>) {
    let mut a = Vec::with_capacity(n);
    for _ in 0..n {
        a.push(gen_random_row(m));
    }
    let mut b_t = Vec::with_capacity(k);
    for _ in 0..k {
        b_t.push(gen_random_row(m));
    }
    (a, b_t)
}

fn show_matrix(a: &Vec<Vec<u64>>) {
    a.iter().for_each(|row| {
        row.iter().for_each(|x| print!("{:4} ", x));
        println!();
    });
}

fn show_matrix_t(a: &Vec<Vec<u64>>, n: usize, m: usize) {
    for i in 0..n {
        for j in 0..m {
            print!("{:4} ", a[j][i])
        }
        println!();
    }
}

unsafe fn mul_vectors(tasks: Vec<(RefU64, RefU64, usize, usize)>, m: usize, t: usize) -> (Vec<u64>, usize) {
    let mut res = Vec::with_capacity(tasks.len());
    for (i, j, ii, jj) in tasks {
        let mut sum = 0;
        for pos in 0..m {
            sum += i.0.offset(pos as isize).read() * j.0.offset(pos as isize).read();
        }
        res.push(sum);
        println!("[{}, {}]={}", ii, jj, sum);
    }

    (res, t)
}

fn update_c(c: &mut Vec<Vec<u64>>, res: (Vec<u64>, usize), n: usize, k: usize, threads: usize) {
    let mut pos = res.1;
    let mut tmp = 0;
    while pos < n * k {
        c[pos / k][pos % k] = res.0[tmp];
        tmp += 1;
        pos += threads;
    }
}

#[derive(Debug, Clone)]
struct RefU64(pub *const u64);
unsafe impl Send for RefU64 {}

fn main() {
    let n = env::args().nth(1).unwrap().parse::<usize>().unwrap();
    let m = env::args().nth(2).unwrap().parse::<usize>().unwrap();
    let k = env::args().nth(3).unwrap().parse::<usize>().unwrap();

    let (a, b_t) = gen_matrices(n, m, k);
    println!("A =");
    // show_matrix(&a);
    println!("B =");
    // show_matrix_t(&b_t, m, k);

    let mut empty_row = Vec::new();
    empty_row.resize(k, 0);
    let mut empty = Vec::new();
    empty.resize(n, empty_row);

    // {
    //     let mut c = empty.clone();
    //     let started_at: Instant = Instant::now();
    //     for i in 0..n {
    //         for j in 0..k {
    //             let mut sum = 0;
    //             unsafe {
    //                 let ptr1 = a[i].as_ptr();
    //                 let ptr2 = b_t[j].as_ptr();
    //                 for t in 0..m {
    //                     sum += ptr1.offset(t as isize).read() * ptr2.offset(t as isize).read();
    //                 }
    //             }
    //             c[i][j] = sum;
    //         }
    //     }
    //     println!("threads 1, took: {:?}, C =", started_at.elapsed());
    //     // show_matrix(&c);
    // }
    // {
    //     let mut c = empty.clone();
    //     let started_at: Instant = Instant::now();
    //     for i in 0..n {
    //         for j in 0..k {
    //             let mut sum = 0;
    //             for t in 0..m {
    //                 sum += a[i][t] * b_t[j][t];
    //             }
    //             c[i][j] = sum
    //         }
    //     }
    //     println!("threads 1, took: {:?}, C =", started_at.elapsed());
    //     // show_matrix(&c);
    // }

    for threads in 1..=(n * k).min(20) {
        let started_at: Instant = Instant::now();
        let mut tasks = Vec::new();
        tasks.resize(threads, Vec::with_capacity((n * k) / threads + 1));
        for i in 0..n {
            for j in 0..k {
                tasks[(i * k + j) % threads].push((RefU64(a[i].as_ptr()), RefU64(b_t[j].as_ptr()), i, j));
            }
        }
        let mut c = empty.clone();
        let task_for_curr_thread = tasks.pop().unwrap();

        let handles: Vec<_> = unsafe {
            tasks
                .clone()
                .into_iter()
                .enumerate()
                .map(|(i, t)| std::thread::spawn(move || mul_vectors(t, m, i)))
                .collect()
        };
        unsafe {
            let res = mul_vectors(task_for_curr_thread, m, threads - 1);
            update_c(&mut c, res, n, k, threads);
        }

        for handle in handles {
            let res = handle.join().unwrap();
            update_c(&mut c, res, n, k, threads);
        }

        println!("threads {}, took: {:?}, C =", threads, started_at.elapsed());
        // show_matrix(&c);
    }
}
