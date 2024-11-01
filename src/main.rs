fn main() {
    println!("Adobe RGB (1998) Matrices by 540THz");
    if let Some(version) = option_env!("VERSION_FROM_GIT_TAG") {
        println!("{}", version);
    }
    if let Some(repourl) = option_env!("GIT_REMOTE_REPO_URL") {
        println!("{}", repourl);
    }
}
