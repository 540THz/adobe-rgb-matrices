use std::process::Command;

fn set_env_version() {
    // delayed initialization: https://blog.m-ou.se/super-let/
    //   "Unfortunately, there is no way to explicitly opt in to temporary lifetime extension.
    //    We’ll have to put a let file in the outermost scope.
    //    The best we can do, today, is by using delayed intialization:"
    // ( via https://www.reddit.com/r/rust/comments/187hbfm/rust_temporary_lifetimes_and_super_let/ )

    // See https://stackoverflow.com/questions/43753491/include-git-commit-hash-as-string-into-rust-program
    // and https://git-scm.com/docs/git-describe
    let s;
    let (tag_num, hash) = if let Ok(o) = Command::new("git").args(["describe", "--tags", "--abbrev=40", "--long"]).output() {
        if !o.status.success() {return}
        // The output is e.g. "v0.0.0-0-gbfd0aee79d3982ad5607a16db1c79af2e2e4183a\n", or empty on error
        s = String::from_utf8(o.stdout).unwrap();
        let p = s.rsplitn(2, '-').collect::<Vec<_>>();
        if p.len() < 2 {return}
        let q = p[1].rsplitn(2, '-').collect::<Vec<_>>();
        if q.len() < 2 {return}
        (if q[0] == "0" {q[1]} else {p[1]}, &p[0][1..41])
    } else {return};

    // See https://stackoverflow.com/questions/3814926/git-commit-date
    // and https://git-scm.com/docs/git-show
    let t;
    let datetime = if let Ok(o) = Command::new("git").args(["show", "-s", "--format=%ci", hash]).output() {
        if !o.status.success() {return}
        // The output is e.g. "2024-10-22 17:00:00 +0900\n"
        t = String::from_utf8(o.stdout).unwrap();
        t.split('\n').next().unwrap()
    } else {return};

    // See https://unix.stackexchange.com/questions/155046/determine-if-git-working-directory-is-clean-from-a-script
    // and https://git-scm.com/docs/git-status
    let modified = if let Ok(o) = Command::new("git").args(["status", "--porcelain"]).output() {
        if !o.status.success() {return}
        let u = String::from_utf8(o.stdout).unwrap();
        if u.is_empty() {""} else {" [modified]"}
    } else {return};

    // See https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-env
    println!("cargo:rustc-env=VERSION_FROM_GIT_TAG={} ({} {}){}", tag_num, &hash[0..8], &datetime[0..10], modified);
}

fn set_env_repourl() {
    // See https://stackoverflow.com/questions/4089430/how-to-determine-the-url-that-a-local-git-repository-was-originally-cloned-from
    let s;
    let url = if let Ok(o) = Command::new("git").args(["config", "--get", "remote.origin.url"]).output() {
        if !o.status.success() {return}
        s = String::from_utf8(o.stdout).unwrap();
        s.split('\n').next().unwrap()
    } else {return};
    // See https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-env
    println!("cargo:rustc-env=GIT_REMOTE_REPO_URL={}", if url.ends_with(".git") {&url[0..url.len()-4]} else {url});
}

fn main() {
    // Set VERSION_FROM_GIT_TAG environment variable if possible
    set_env_version();
    // Set GIT_REMOTE_REPO_URL environment variable if possible
    set_env_repourl();
    // Recompile after git add/reset/checkout/commit/tag/...
    // See https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/heads");
    println!("cargo:rerun-if-changed=.git/refs/tags");
}
