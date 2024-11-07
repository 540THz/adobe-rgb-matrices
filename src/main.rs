// #![allow(unused)]

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

fn diag(v: &Vec<BigRational>) -> Vec<Vec<BigRational>> {
    (0..v.len()).map(|i|
        (0..v.len()).map(|j|
            if i == j {v[i].clone()} else {BigRational::zero()}
        ).collect()
    ).collect()
}

#[inline(never)]
fn compute_primary_matrix(xy_red: &str, xy_green: &str, xy_blue: &str, xy_white: &str) -> Vec<Vec<BigRational>> {
    // See https://cmykspot.blogspot.com/2024/10/xyz-of-adobe-rgb-primaries.html (Korean)
    // or  https://mina86.com/2019/srgb-xyz-matrix/ (English)

    let one = BigRational::one();
    let f = |it: &mut std::str::Split<'_, char>| {
        let x = BigRational::from_str(it.next().unwrap()).unwrap();
        let y = BigRational::from_str(it.next().unwrap()).unwrap();
        let z = &one - &x - &y;
        vec![&x / &y, one.clone(), &z / &y]
    };

    let mt = [xy_red.split(','), xy_green.split(','), xy_blue.split(',')].iter_mut().map(f).collect();
    let bt = [xy_white.split(',')].iter_mut().map(f).collect();
    let yt = multiply(&bt, &inverse(&mt));

    let m = transpose(&mt);
    let d = diag(&yt[0]);
    multiplyabt(&m, &d)
}

fn get_matrix<V: AsRef<[&'static str]>>(arr_2d: &[V]) -> Vec<Vec<BigRational>> {
    // See https://medium.com/@trunghuynh/why-rust-is-harder-than-java-part-1-2d-array-argument-c026216c28f8
    // and https://users.rust-lang.org/t/mutidimensional-array-as-parameter/27941/12
    arr_2d.iter().map(|v|
        v.as_ref().iter().map(|&s|
            BigRational::from_str(s).unwrap()
        ).collect()
    ).collect()
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

#[inline(never)]
fn to_string_denom(a: &Vec<Vec<BigRational>>, denom: u64) -> String {
    let bigd = BigRational::from_u64(denom).unwrap();
    to_string_f(a, |x| format!("{}/{}", x * &bigd, &bigd))
}

#[inline(never)]
fn to_string_decimal(a: &Vec<Vec<BigRational>>, dp: i32) -> String {
    let zero = BigRational::zero();
    let ten  = BigRational::from_i32(10).unwrap();
    to_string_f(a, |x| {
        let mut t = if x < &zero {-x} else {x.clone()};
        let n = t.to_integer();
        let size = if x < &zero {1} else {0}
                 + (n.bits() as f64 * 0.3010299956639812) as usize + 1
                 + if dp > 0 {dp as usize + 1} else {0};
        let mut s = String::with_capacity(size);
        write!(s, "{}{}{}", if x < &zero {"-"} else {""},  // sign
                            n,  // integral part, not including the sign
                            if dp > 0 {"."} else {""}).unwrap();  // decimal point
        for _ in 0..dp {
            t = t.fract() * &ten;
            s.push((b'0' + t.to_u8().unwrap()) as char);
        }
        s
    })
}

#[inline(never)]
fn is_exact(a: &Vec<Vec<BigRational>>, dp: i32) -> bool {
    let sc = BigRational::from_i32(10).unwrap().pow(dp);
    a.iter().all(|v| v.iter().all(|x| (x * &sc).is_integer()))
}

fn print_values(a: &Vec<Vec<BigRational>>, denom: Option<u64>, dp: Option<i32>, extra: &str) {
    let s = match denom {
        Some(d) => to_string_denom(a, d),
        None    => to_string(a),
    };
    let dp = dp.unwrap_or(20);
    let eq_char = if is_exact(a, dp) {'='} else {'≈'};
    print!("  {}\n{} {}\n{}", s, eq_char, to_string_decimal(a, dp), extra);
}

fn print_result1(no: i32, name: &str, desc: &str, def: &str, desc2: &str, a: &Vec<Vec<BigRational>>, denom: Option<u64>, dp: Option<i32>, extra: &str) {
    print!("({}) \x1b[1;92m[{}]\x1b[0m: \x1b[1;96m{}\x1b[0m matrix", no, name, desc); // green, cyan
    if !def.is_empty() {print!(" = \x1b[93m{}\x1b[0m", def);} // yellow
    if !desc2.is_empty() {
        let n = no.to_string().len() + name.chars().count()
              + desc.chars().count() + def.chars().count()
              + if def.is_empty() {14} else {17};
        print!("{: <2$} ( {} )", "", desc2, if n < 56 {56 - n} else {0});
    }
    println!();
    print_values(a, denom, dp, extra);
}

#[inline(never)]
fn do_main() {
    // [P]: RGB -> XYZ matrix (1st col = XYZ_red, 2nd col = XYZ_green, 3rd col = XYZ_blue)
    let xyz_from_rgb = compute_primary_matrix(
        "64/100,33/100", "21/100,71/100", "15/100,6/100", "3127/10000,3290/10000"
    );
    assert_eq!(&xyz_from_rgb, &get_matrix(&[
        ["573536/994567",   "263643/1420810",  "187206/994567" ],
        ["591459/1989134", "6239551/9945670",  "374412/4972835"],
        [ "53769/1989134",  "351524/4972835", "4929758/4972835"],
    ]));
    // [P inv]: XYZ -> RGB matrix
    let rgb_from_xyz = inverse(&xyz_from_rgb);

    // [C]: XYZ -> LMS matrix (Linearized Bradford CAT matrix)
    let lms_from_xyz = get_matrix(&[
        [ "8951/10000",  "2664/10000", "-1614/10000"],
        ["-7502/10000", "17135/10000",   "367/10000"],
        [  "389/10000",  "-685/10000", "10296/10000"],
    ]);
    // [C inv]: LMS -> XYZ matrix
    let xyz_from_lms = inverse(&lms_from_xyz);

    println!(concat!(
        "## [C], [●], [○], and [Q1]: decimals are exact.\n",
        "## All other matrices: decimals are the TRUNC'ated (aka ROUNDDOWN'ed) APPROXIMATIONS to 20 decimal places.\n",
    ));

    println!(concat!("\x1b[90m",
        ":: [P] is computed from the xy chromaticity coordinates of primary colors and white.\n",
        "::             red   green   blue   white (D65)\n",
        "::         x   0.64   0.21   0.15   0.3127\n",
        "::         y   0.33   0.71   0.06   0.3290\n",
        ":: See \"Adobe RGB (1998) Color Image Encoding (May 2005)\" 4.3.1.1 (p.10).\n",
    "\x1b[0m"));
    print_result1(1,  "P",     "RGB -> XYZ", "", "", &xyz_from_rgb, None, None, "");
    print_result1(2,  "P inv", "XYZ -> RGB", "", "", &rgb_from_xyz, None, None, "\n");

    println!(concat!("\x1b[90m",
        ":: [C] is the linearized Bradford CAT, used in LLAB and original CIECAM97s.\n",
        ":: See \"ICC.1:2001-04\" E.1.2 (p.88).\n",
        "\n",
        ":: (EXACT DECIMALS) Every number in [C] has at most 4 decimal places.\n",
    "\x1b[0m"));
    print_result1(3,  "C",     "XYZ -> LMS", "", "", &lms_from_xyz, Some(10000), Some(4), "");
    print_result1(4,  "C inv", "LMS -> XYZ", "", "", &xyz_from_lms, None,        None,    "\n");

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
    println!();
    do_main();
}
