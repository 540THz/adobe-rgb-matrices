fn main() {
    println!("Adobe RGB (1998) Matrices by 540THz");
    if let Some(version) = option_env!("GIT_HASH_VERSION") {
        print!("Version {}\n", version);
    }
}
