// #![allow(unused_imports, unused_variables, dead_code)]

use std::io::IsTerminal;
use std::str::FromStr;
use num::{FromPrimitive, ToPrimitive};
use num::{Zero, One};

use anstream::{print, println, ColorChoice};
use num::rational::BigRational;

fn vscale(v: &Vec<BigRational>, sc: &BigRational) -> Vec<BigRational> {
    v.iter().map(|x| x * sc).collect()
}

fn vscaleadd(v: &Vec<BigRational>, sc: &BigRational, w: &Vec<BigRational>) -> Vec<BigRational> {
    v.iter().zip(w.iter()).map(|(x, y)| x * sc + y).collect()
}

fn scale(a: &Vec<Vec<BigRational>>, sc: &BigRational) -> Vec<Vec<BigRational>> {
    a.iter().map(|v| vscale(v, sc)).collect()
}

fn scaleround(a: &Vec<Vec<BigRational>>, sc: &BigRational) -> Vec<Vec<BigRational>> {
    a.iter().map(|v|
        v.iter().map(|x| (x * sc).round()).collect()
    ).collect()
}

fn round(a: &Vec<Vec<BigRational>>, denom: u64) -> Vec<Vec<BigRational>> {
    let sc = BigRational::from_u64(denom).unwrap();
    let scr = sc.recip();
    scale(&scaleround(&a, &sc), &scr)
}

fn sumcolumns(a: &Vec<Vec<BigRational>>) -> Vec<Vec<BigRational>> {
    a.iter().map(|v| vec![v.iter().sum()]).collect()
}

fn transpose(a: &Vec<Vec<BigRational>>) -> Vec<Vec<BigRational>> {
    (0..a[0].len()).map(|j|
        (0..a.len()).map(|i| a[i][j].clone()).collect()
    ).collect()
}

fn multiply(a: &Vec<Vec<BigRational>>, b: &Vec<Vec<BigRational>>) -> Vec<Vec<BigRational>> {
    let bt = transpose(&b);
    a.iter().map(|v|
        bt.iter().map(|w|
            v.iter().zip(w.iter()).map(|(x, y)| x * y).sum()
        ).collect()
    ).collect()
}

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
            if ma[j][i] != BigRational::zero() {
                // (1) Swap i-th and j-th rows to get a nonzero ma[i][i]
                if j != i {ma.swap(i, j); id.swap(i, j);}
                break;
            }
        }
        assert_ne!(ma[i][i], BigRational::zero());
        for j in i+1..n {
            // (3) Add -(ma[j][i]/ma[i][i]) multiple of i-th row to j-th row
            let sc = -(&ma[j][i] / &ma[i][i]);
            ma[j] = vscaleadd(&ma[i], &sc, &ma[j]);
            id[j] = vscaleadd(&id[i], &sc, &id[j]);
        }
    }

    // BACK SUBSTITUTION
    for i in (0..n).rev() {  // from n-1 to 0
        for j in (0..i).rev() {  // from i-1 to 0
            // Same as (3) above 
            let sc = -(&ma[j][i] / &ma[i][i]);
            ma[j] = vscaleadd(&ma[i], &sc, &ma[j]);
            id[j] = vscaleadd(&id[i], &sc, &id[j]);
        }
        // (2) Multiply i-th row by 1/ma[i][i] 
        let sc = ma[i][i].recip();
        ma[i] = vscale(&ma[i], &sc);
        id[i] = vscale(&id[i], &sc);
    }

    id
}

fn get_primary_matrix(xy_red: &str, xy_green: &str, xy_blue: &str, xy_white: &str) -> Vec<Vec<BigRational>> {
    // See https://cmykspot.blogspot.com/2024/10/xyz-of-adobe-rgb-primaries.html (Korean)
    // or  https://mina86.com/2019/srgb-xyz-matrix/ (English)

    let one = BigRational::one();
    // split() returns an iterator
    let (mut it_r, mut it_g, mut it_b, mut it_w) = (xy_red.split(','), xy_green.split(','), xy_blue.split(','), xy_white.split(','));
    let (x_red  , y_red  ) = (BigRational::from_str(it_r.next().unwrap()).unwrap(), BigRational::from_str(it_r.next().unwrap()).unwrap());
    let (x_green, y_green) = (BigRational::from_str(it_g.next().unwrap()).unwrap(), BigRational::from_str(it_g.next().unwrap()).unwrap());
    let (x_blue , y_blue ) = (BigRational::from_str(it_b.next().unwrap()).unwrap(), BigRational::from_str(it_b.next().unwrap()).unwrap());
    let (x_white, y_white) = (BigRational::from_str(it_w.next().unwrap()).unwrap(), BigRational::from_str(it_w.next().unwrap()).unwrap());
    let z_red   = &one - &x_red   - &y_red;
    let z_green = &one - &x_green - &y_green;
    let z_blue  = &one - &x_blue  - &y_blue;
    let z_white = &one - &x_white - &y_white;

    let m = vec![
        vec![&x_red / &y_red, &x_green / &y_green, &x_blue / &y_blue],
        vec![one.clone(), one.clone(), one.clone()],
        vec![&z_red / &y_red, &z_green / &y_green, &z_blue / &y_blue],
    ];
    let b = vec![
        vec![&x_white / &y_white],
        vec![one.clone()],
        vec![&z_white / &y_white],
    ];

    // Y_red·[(1st col of M)] + Y_green·[(2st col of M)] + Y_blue·[(3rd col of M)] = [b]
    //
    // So we have the matrix equation  [M]·[y] = [b]     (1)
    // where [y] be a 3x1 matrix, y_11 = Y_red, y_21 = Y_green, and y_31 = Y_blue.
    //
    // The solution to (1) is   [y] = [M inv]·[b]   if [M] is invertible.

    let y = multiply(&inverse(&m), &b);
    // y[0][0] == Y_red / y[1][0] == Y_green / y[2][0] == Y_blue

    // returns [Y_red·(1st col of M)  Y_green·(2st col of M)  Y_blue·(3rd col of M)]
    vec![
        vec![&m[0][0] * &y[0][0], &m[0][1] * &y[1][0], &m[0][2] * &y[2][0]],
        vec![&m[1][0] * &y[0][0], &m[1][1] * &y[1][0], &m[1][2] * &y[2][0]],
        vec![&m[2][0] * &y[0][0], &m[2][1] * &y[1][0], &m[2][2] * &y[2][0]],
    ]
}

fn fmatrixf(a: &Vec<Vec<BigRational>>, f: impl FnMut(&BigRational) -> String + Clone) -> String {
    format!("[{}]", a.iter().map(|v|
        format!("[{}]", v.iter().map(f.clone()).collect::<Vec<_>>().join(", "))
    ).collect::<Vec<_>>().join(", "))
}

fn fmatrix(a: &Vec<Vec<BigRational>>) -> String {
    fmatrixf(a, |x| x.to_string())
}

fn fmatrix_samedenom(a: &Vec<Vec<BigRational>>, denom: u64) -> String {
    let bigd = BigRational::from_u64(denom).unwrap();
    fmatrixf(a, |x| format!("{}/{}", x * &bigd, &bigd))
}

fn fmatrix_rounddown(a: &Vec<Vec<BigRational>>, decimal_places: usize) -> String {
    fmatrixf(a, |x| {
        let zero = BigRational::zero();
        let ten  = BigRational::from_i32(10).unwrap();
        let mut s = String::with_capacity(32);
        let mut t = if x < &zero {s.push('-'); -x} else {x.clone()};
        s.push_str(&t.to_integer().to_string());
        s.push('.');
        for _ in 0..decimal_places {
            t = t.fract() * &ten;
            s.push((b'0' + t.to_u8().unwrap()) as char);
        }
        s
    })
}

fn print_result1(no: i32, name: &str, desc: &str, def: &str, desc2: &str, value: &String, extra: &str) {
    print!("({}) \x1b[1;92m[{}]\x1b[0m: \x1b[1;96m{}\x1b[0m matrix", no, name, desc); // green, cyan
    if !def.is_empty() {print!(" = \x1b[93m{}\x1b[0m", def);} // yellow
    if !desc2.is_empty() {
        let n = no.to_string().len() + name.chars().count()
                     + desc.chars().count() + def.chars().count()
                     + if def.is_empty() {14} else {17};
        print!("{: <2$} ( {} )", "", desc2, if n < 56 {56 - n} else {0});
    }
    println!("\n  {}{}", value, extra);
}

fn print_result2(sym: &str, name: &str, desc: &str, def: &str, value: &String, extra: &str) {
    print!("{} \x1b[1;96m{}\x1b[0m ({})", sym, name, desc); // cyan
    if !def.is_empty() {print!(" = \x1b[93m{}\x1b[0m", def);} // yellow
    println!("\n  {}{}", value, extra);
}

fn adobe_rgb_matrices() {
    // [P]: RGB -> XYZ matrix
    //  - calculated from xy chromaticity coordinates of primary colors (red, green, blue) and white
    //  - See Adobe RGB (1998) Color Image Encoding (May 2005) 4.3.1.1 (p.88)
    let xyz_from_rgb = get_primary_matrix(
        "64/100,33/100", "21/100,71/100", "15/100,6/100", "3127/10000,3290/10000"
    );
    assert_eq!(&xyz_from_rgb, &vec![
        vec![BigRational::from_str("573536/994567").unwrap(),  BigRational::from_str("263643/1420810").unwrap(),  BigRational::from_str("187206/994567").unwrap()],
        vec![BigRational::from_str("591459/1989134").unwrap(), BigRational::from_str("6239551/9945670").unwrap(), BigRational::from_str("374412/4972835").unwrap()],
        vec![BigRational::from_str("53769/1989134").unwrap(),  BigRational::from_str("351524/4972835").unwrap(),  BigRational::from_str("4929758/4972835").unwrap()],
    ]);
    // [P inv]: XYZ -> RGB matrix
    let rgb_from_xyz = inverse(&xyz_from_rgb);

    // [C]: XYZ -> LMS matrix
    //  - Linearized Bradford CAT / See ICC.1:2001-04 E.1.2 (p.88)
    let lms_from_xyz = vec![
        vec![BigRational::from_str("8951/10000").unwrap(),  BigRational::from_str("2664/10000").unwrap(),  BigRational::from_str("-1614/10000").unwrap()],
        vec![BigRational::from_str("-7502/10000").unwrap(), BigRational::from_str("17135/10000").unwrap(), BigRational::from_str("367/10000").unwrap()],
        vec![BigRational::from_str("389/10000").unwrap(),   BigRational::from_str("-685/10000").unwrap(),  BigRational::from_str("10296/10000").unwrap()],
    ];
    // [C inv]: LMS -> XYZ matrix
    let xyz_from_lms = inverse(&lms_from_xyz);

    // ■ XYZ of 'white (D65)'
    //  - equals sum of the columns of [P]
    //  - 3127/3290 3290/3290 3583/3290
    let xyz_white = sumcolumns(&xyz_from_rgb);
    // ● XYZ of 'PCS white (D50)'
    //  - icc profile header - offset 0x44, length 0xC / See ICC.1:2021-04 6.1 (p.12)
    //  - 63190/65536 65536/65536 54061/65536 (0xF6D6 0x10000 0xD32D) from AdobeRGB1998.icc
    let xyz_pcs_white = vec![
        vec![BigRational::from_str("63190/65536").unwrap()],
        vec![BigRational::from_str("65536/65536").unwrap()],
        vec![BigRational::from_str("54061/65536").unwrap()],
    ];

    // □ LMS of 'white (D65)'     = [C][■]
    // ○ LMS of 'PCS white (D50)' = [C][●]
    let lms_white     = multiply(&lms_from_xyz, &xyz_white);
    let lms_pcs_white = multiply(&lms_from_xyz, &xyz_pcs_white);

    // [Λ]:     LMS -> PCSLMS matrix = diag(○/□)
    // [Λ inv]: PCSLMS -> LMS matrix = diag(□/○)
    let (pcslms_from_lms, lms_from_pcslms) = {
        (0..3).map(|i|
            (0..3).map(|j|
                if i == j {let x = &lms_white[i][0] / &lms_pcs_white[i][0]; (x.recip(), x)}
                else      {let x = BigRational::zero();                     (x.clone(), x)}
            ).collect()
        ).collect()
    };
    assert_eq!(&lms_from_pcslms, &inverse(&pcslms_from_lms));

    // [A]:     XYZ -> PCSXYZ matrix ( XYZ -> LMS -> PCSLMS -> PCSXYZ ) = [C inv][Λ][C]
    // [A inv]: PCSXYZ -> XYZ matrix ( PCSXYZ -> PCSLMS -> LMS -> XYZ ) = [C inv][Λ inv][C]
    let pcsxyz_from_xyz = multiply(&multiply(&xyz_from_lms, &pcslms_from_lms), &lms_from_xyz);
    let xyz_from_pcsxyz = multiply(&multiply(&xyz_from_lms, &lms_from_pcslms), &lms_from_xyz);
    assert_eq!(&xyz_from_pcsxyz, &inverse(&pcsxyz_from_xyz));

    // [Q]:     RGB -> PCSXYZ matrix ( RGB -> XYZ -> PCSXYZ ) = [A][P]         = [C inv][Λ][C][P]
    // [Q inv]: PCSXYZ -> RGB matrix ( PCSXYZ -> XYZ -> RGB ) = [P inv][A inv] = [P inv][C inv][Λ inv][C]
    let pcsxyz_from_rgb = multiply(&pcsxyz_from_xyz, &xyz_from_rgb);
    let rgb_from_pcsxyz = multiply(&rgb_from_xyz, &xyz_from_pcsxyz);
    assert_eq!(&rgb_from_pcsxyz, &inverse(&pcsxyz_from_rgb));

    // [Q1]:     RGB -> PCSXYZ1 matrix ( rounded values of [Q] to the nearest multiples of 1/65536, ties away from zero )
    // [Q1 inv]: PCSXYZ1 -> RGB matrix ( inverse of [Q1] )
    let pcsxyz1_from_rgb = round(&pcsxyz_from_rgb, 65536);
    let rgb_from_pcsxyz1 = inverse(&pcsxyz1_from_rgb);

    println!("## The decimals in [C], [●], [○], and [Q1] are exact.");
    println!("## Other decimals are the ROUNDDOWN'ed APPROXIMATIONS to 20 decimal places.");
    println!();

    println!("\x1b[90m:: [P] is calculated from the xy chromaticity coordinates of primary colors and white.\x1b[0m");
    println!("\x1b[90m::             red   green   blue   white (D65)\x1b[0m");
    println!("\x1b[90m::         x   0.64   0.21   0.15   0.3127\x1b[0m");
    println!("\x1b[90m::         y   0.33   0.71   0.06   0.3290\x1b[0m");
    println!("\x1b[90m:: See \"Adobe RGB (1998) Color Image Encoding (May 2005)\" 4.3.1.1 (p.10).\x1b[0m");
    println!();
    let v = format!("{}\n≈ {}", fmatrix(&xyz_from_rgb), fmatrix_rounddown(&xyz_from_rgb, 20));
    let w = format!("{}\n≈ {}", fmatrix(&rgb_from_xyz), fmatrix_rounddown(&rgb_from_xyz, 20));
    print_result1(1,  "P",     "RGB -> XYZ", "", "", &v, "");
    print_result1(2,  "P inv", "XYZ -> RGB", "", "", &w, "\n");

    println!("\x1b[90m:: [C] is the linearized Bradford CAT, used in LLAB and original CIECAM97s.\x1b[0m");
    println!("\x1b[90m:: See \"ICC.1:2001-04\" E.1.2 (p.88).\x1b[0m");
    println!("\x1b[90m:: The values of [C] can be expressed as EXACT DECIMALS with 4 decmial places.\x1b[0m");
    println!();
    let v = format!("{}\n= {}", fmatrix_samedenom(&lms_from_xyz, 10000), fmatrix_rounddown(&lms_from_xyz, 4));
    let w = format!("{}\n≈ {}", fmatrix(&xyz_from_lms),                  fmatrix_rounddown(&xyz_from_lms, 20));
    print_result1(3,  "C",     "XYZ -> LMS", "", "", &v, "");
    print_result1(4,  "C inv", "LMS -> XYZ", "", "", &w, "\n");

    println!("\x1b[90m:: [■] is the sum of the columns of [P].\x1b[0m");
    println!();
    println!("\x1b[90m:: [●] is from the ICC profile header, offset 0x44, length 0xC.\x1b[0m");
    println!("\x1b[90m:: See \"ICC.1:2001-04\" 6.1 (p.12) and A.1 (p.64).\x1b[0m");
    println!("\x1b[90m:: The values of [●] can be expressed as EXACT DECIMALS with 16 decmial places.\x1b[0m");
    println!();
    println!("\x1b[90m:: The values of [○] can be expressed as EXACT DECIMALS with 20 decmial places.\x1b[0m");
    println!();
    let v = format!("{}\n≈ {}", fmatrix_samedenom(&xyz_white, 3290),      fmatrix_rounddown(&xyz_white, 20));
    let w = format!("{}\n= {}", fmatrix_samedenom(&xyz_pcs_white, 65536), fmatrix_rounddown(&xyz_white, 16));
    print_result2("■", "XYZ of white",     "D65", "", &v, "");
    print_result2("●", "XYZ of PCS white", "D50", "", &w, "");
    let v = format!("{}\n≈ {}", fmatrix(&lms_white),     fmatrix_rounddown(&lms_white, 20));
    let w = format!("{}\n= {}", fmatrix(&lms_pcs_white), fmatrix_rounddown(&lms_pcs_white, 20));
    print_result2("□", "LMS of white",     "D65", "[C]·[■]", &v, "");
    print_result2("○", "LMS of PCS white", "D50", "[C]·[●]", &w, "\n");

    let v = format!("{}\n≈ {}", fmatrix(&pcslms_from_lms), fmatrix_rounddown(&pcslms_from_lms, 20));
    let w = format!("{}\n≈ {}", fmatrix(&lms_from_pcslms), fmatrix_rounddown(&lms_from_pcslms, 20));
    print_result1(5,  "Λ",     "LMS -> PCSLMS", "diag(○/□)", "", &v, "");
    print_result1(6,  "Λ inv", "PCSLMS -> LMS", "diag(□/○)", "", &w, "\n");

    let v = format!("{}\n≈ {}", fmatrix(&pcsxyz_from_xyz), fmatrix_rounddown(&pcsxyz_from_xyz, 20));
    let w = format!("{}\n≈ {}", fmatrix(&xyz_from_pcsxyz), fmatrix_rounddown(&xyz_from_pcsxyz, 20));
    print_result1(7,  "A",     "XYZ -> PCSXYZ", "[C inv]·[Λ]·[C]",     "XYZ -> LMS -> PCSLMS -> PCSXYZ", &v, "");
    print_result1(8,  "A inv", "PCSXYZ -> XYZ", "[C inv]·[Λ inv]·[C]", "PCSXYZ -> PCSLMS -> LMS -> XYZ", &w, "\n");

    let v = format!("{}\n≈ {}", fmatrix(&pcsxyz_from_rgb), fmatrix_rounddown(&pcsxyz_from_rgb, 20));
    let w = format!("{}\n≈ {}", fmatrix(&rgb_from_pcsxyz), fmatrix_rounddown(&rgb_from_pcsxyz, 20));
    print_result1(9,  "Q",     "RGB -> PCSXYZ", "[A]·[P]",         "RGB -> XYZ -> PCSXYZ", &v, "");
    print_result1(10, "Q inv", "PCSXYZ -> RGB", "[P inv]·[A inv]", "PCSXYZ -> XYZ -> RGB", &w, "\n");

    println!("\x1b[90m:: [Q1] is the matrix obtained by rounding the values of [Q] to the nearest multiples of 1/65536, ties away from zero.\x1b[0m");
    println!("\x1b[90m:: It is EXACTLY THE SAME as [rXYZ gXYZ bXYZ] in AdobeRGB1998.icc.\x1b[0m");
    println!("\x1b[90m:: The sum of the columns of [Q1] equals [●], i.e., XYZ of PCS white (D50).\x1b[0m");
    println!("\x1b[90m:: The values of [Q1] can be expressed as EXACT DECIMALS with 16 decimal places.\x1b[0m");
    println!();
    println!("\x1b[90m:: [Q1 inv] is the inverse of [Q1], literally.\x1b[0m");
    println!("\x1b[90m:: It is NOT obtained by rounding the values of [Q inv].\x1b[0m");
    println!();
    let v = format!("{}\n= {}", fmatrix_samedenom(&pcsxyz1_from_rgb, 65536), fmatrix_rounddown(&pcsxyz1_from_rgb, 16));
    let w = format!("{}\n≈ {}", fmatrix(&rgb_from_pcsxyz1),                  fmatrix_rounddown(&rgb_from_pcsxyz1, 20));
    print_result1(11, "Q1",     "RGB -> PCSXYZ1", "", "", &v, "");
    print_result1(12, "Q1 inv", "PCSXYZ1 -> RGB", "", "", &w, "\n");

}

fn main() {
    if !std::io::stdout().is_terminal() {
        ColorChoice::Never.write_global();
    }
    println!("Adobe RGB (1998) Matrices by 540THz");
    if let Some(version) = option_env!("GIT_HASH_VERSION") {
        print!("Version {}\n", version);
    }
    println!();
    adobe_rgb_matrices();
}
