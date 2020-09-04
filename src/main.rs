use std::process::Command;
use std::path::PathBuf;
// use toml
fn main() {
    struct Package {
        name: String,
        version: String,
        description: Option<String>,
        authors: Vec<String>,
        keywords: Option<Vec<String>>,
        repository: Option<String>,
        homepage: Option<String>,
        license: Option<String>,
    }
    let command = String::from_utf8(Command::new("cargo").arg("locate-project").output().unwrap().stdout).unwrap();
    let mut vec = command.split("/").collect::<Vec<&str>>().split_off(1);
    let path_as_vec = vec.split_last().unwrap().1;

    let path: PathBuf = path_as_vec.iter().collect();
    
    println!("{:?}", path);
}
