use std::process::Command;

fn main() {
    // See https://stackoverflow.com/questions/43753491/include-git-commit-hash-as-string-into-rust-program
    // and https://git-scm.com/docs/git-describe
    if let Ok(o) = Command::new("git").args(["describe", "--tags", "--abbrev=40", "--long"]).output() {
        if !o.status.success() {return;}
        let s = String::from_utf8(o.stdout).unwrap();
        let (ver, hash) = {
            let pts: Vec<_> = s.rsplitn(3, '-').collect();
            if pts.len() < 3 {return;}
            (if pts[1] == "0" {pts[2]} else {&*format!("{}-{}", pts[2], pts[1])}, &pts[0][1..41])
        };
        // See https://stackoverflow.com/questions/3814926/git-commit-date
        if let Ok(o) = Command::new("git").args(["show", "-s", "--format=%ci", hash]).output() {
            if !o.status.success() {return;}
            let s = String::from_utf8(o.stdout).unwrap();
            let datetime = s.split('\n').next().unwrap();
            println!("cargo:rustc-env=GIT_HASH_VERSION={} ({} {})", ver, &hash[0..8], &datetime[0..10]);
        }
    }
}
