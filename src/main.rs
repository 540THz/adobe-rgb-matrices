#![allow(unused)]

use std::fmt::Write;
use std::io::IsTerminal;
use std::str::FromStr;

use anstream::{print, println, ColorChoice};
use num::rational::BigRational;
use num::{Zero, One, FromPrimitive, ToPrimitive};

fn rowscale(a: &mut Vec<Vec<BigRational>>, i: usize, sc: &BigRational) {
    (0..a[i].len()).for_each(|k| {a[i][k] *= sc;});
}

fn rowscaleadd(a: &mut Vec<Vec<BigRational>>, i: usize, sc: &BigRational, j: usize) {
    (0..a[i].len()).for_each(|k| {let tmp = &a[i][k] * sc; a[j][k] += tmp;});
}

#[inline(never)]
fn identity(n: usize) -> Vec<Vec<BigRational>> {
    (0..n).map(|i|
        (0..n).map(|j|
            if i == j {BigRational::one()} else {BigRational::zero()}
        ).collect()
    ).collect()
}

fn inverse(a: &Vec<Vec<BigRational>>) -> Vec<Vec<BigRational>> {
    let n = a.len();
    let mut ma = a.clone();
    let mut id = identity(n);

    // Elementary row operations
    //  (1) Interchanging two rows
    //  (2) Multiplying a row by a non-zero scalar
    //  (3) Adding a scalar multiple of one row to another
    // https://en.wikipedia.org/wiki/Gaussian_elimination#Row_operations
    // https://en.wikipedia.org/wiki/Gaussian_elimination#Finding_the_inverse_of_a_matrix

    // FORWARD ELIMINATION
    for i in 0..n {
        for j in i..n {
            if !ma[j][i].is_zero() {
                // (1) Swap i-th and j-th rows to get a nonzero ma[i][i]
                if j != i {ma.swap(i, j); id.swap(i, j);}
                break;
            }
        }
        assert!(!ma[i][i].is_zero());
        for j in i+1..n {
            // (3) Add -(ma[j][i]/ma[i][i]) multiple of i-th row to j-th row
            let sc = -(&ma[j][i] / &ma[i][i]);
            rowscaleadd(&mut ma, i, &sc, j);
            rowscaleadd(&mut id, i, &sc, j);
        }
    }

    // BACK SUBSTITUTION
    for i in (0..n).rev() {  // from n-1 to 0
        for j in (0..i).rev() {  // from i-1 to 0
            // Same as (3) above
            let sc = -(&ma[j][i] / &ma[i][i]);
            rowscaleadd(&mut ma, i, &sc, j);
            rowscaleadd(&mut id, i, &sc, j);
        }
        // (2) Multiply i-th row by 1/ma[i][i]
        let sc = ma[i][i].recip();
        rowscale(&mut ma, i, &sc);
        rowscale(&mut id, i, &sc);
    }

    id
}

fn transpose(a: &Vec<Vec<BigRational>>) -> Vec<Vec<BigRational>> {
    (0..a[0].len()).map(|j|
        (0..a.len()).map(|i| a[i][j].clone()).collect()
    ).collect()
}

fn multiplyabt(a: &Vec<Vec<BigRational>>, b: &Vec<Vec<BigRational>>) -> Vec<Vec<BigRational>> {
    a.iter().map(|v|
        b.iter().map(|w|
            v.iter().zip(w.iter()).map(|(x, y)| x * y).sum()
        ).collect()
    ).collect()
}

fn multiply(a: &Vec<Vec<BigRational>>, b: &Vec<Vec<BigRational>>) -> Vec<Vec<BigRational>> {
    multiplyabt(a, &transpose(b))
}

#[inline(never)]
fn to_string_f(a: &Vec<Vec<BigRational>>, f: impl FnMut(&BigRational) -> String + Clone) -> String {
    format!("[{}]", a.iter().map(|v|
        format!("[{}]", v.iter().map(f.clone()).collect::<Vec<_>>().join(", "))
    ).collect::<Vec<_>>().join(", "))
}

#[inline(never)]
fn to_string(a: &Vec<Vec<BigRational>>) -> String {
    to_string_f(a, |x| x.to_string())
}

fn main() {
    if !std::io::stdout().is_terminal() {
        ColorChoice::Never.write_global();
    }
    println!("Adobe RGB (1998) Matrices by 540THz");
    if let Some(version) = option_env!("VERSION_FROM_GIT_TAG") {
        println!("{}", version);
    }
    if let Some(repourl) = option_env!("GIT_REMOTE_REPO_URL") {
        println!("{}", repourl);
    }

    // One example of creating a matrix and its inverse
    // Source: https://web.archive.org/web/20200815053323/https://nhigham.com/2019/01/23/who-invented-the-matrix-condition-number/
    let a = vec![
        vec![BigRational::from_i32(-149).unwrap(), BigRational::from_i32(-50).unwrap(), BigRational::from_i32(-154).unwrap()],
        vec![BigRational::from_i32(537).unwrap(),  BigRational::from_i32(180).unwrap(), BigRational::from_i32(546).unwrap() ],
        vec![BigRational::from_i32(-27).unwrap(),  BigRational::from_i32(-9).unwrap(),  BigRational::from_i32(-25).unwrap() ],
    ];
    let a_inv = inverse(&a);
    println!("\x1b[1;92m[A]\x1b[0m     = {}", to_string(&a));
    println!("\x1b[1;92m[A inv]\x1b[0m = {}", to_string(&a_inv));
    let i_3 = multiply(&a, &a_inv);
    println!("\x1b[1;96m[A]·[A inv]\x1b[0m = {}", to_string(&i_3));
    assert_eq!(i_3, identity(3));

    // Another example of creating a matrix and its inverse
    // Source: https://en.wikipedia.org/wiki/Hilbert_matrix
    let n: usize = 5;
    let h = (0..n).map(|i|
        (0..n).map(|j|
            BigRational::from_str(&format!("1/{}", i + j + 1)).unwrap()
        ).collect()
    ).collect();
    let h_inv = inverse(&h);
    println!("\x1b[1;92m[H]\x1b[0m     = {}", to_string(&h));
    println!("\x1b[1;92m[H inv]\x1b[0m = {}", to_string(&h_inv));
    let i_n = multiply(&h, &h_inv);
    println!("\x1b[1;96m[H]·[H inv]\x1b[0m = {}", to_string(&i_n));
    assert_eq!(i_n, identity(n));

}
