use dialoguer::{theme::ColorfulTheme, Input};
use serde_derive;
use std::fs;
use std::io::Read;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;
use toml;
#[derive(serde_derive::Deserialize, Debug)]
struct Package {
    name: String,                //done
    version: String,             //done
    description: Option<String>, //done
    authors: Vec<String>,
    keywords: Option<Vec<String>>,
    repository: Option<String>,
    homepage: Option<String>, //done
    license: Option<String>,  //done
}
#[derive(serde_derive::Deserialize, Debug)]
struct RustPackage {
    package: Package,
    cargo_aur: Option<Config>,
}
#[derive(serde_derive::Deserialize, Debug)]
struct Config {}
fn main() {
    let command = String::from_utf8(
        Command::new("cargo")
            .arg("locate-project")
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    // some hacky way to get the current rust project directory
    let vec = command.split("/").collect::<Vec<&str>>().split_off(1);
    let path_as_vec = vec.split_last().unwrap().1;
    let mut path: PathBuf = path_as_vec.iter().collect();
    path.push("Cargo.toml");
    let path = PathBuf::from("/").join(path);
    // here we read the cargo.toml file into a string. and then we deserialize it into the RustPackage struct
    let mut file = String::new();
    fs::File::open(path)
        .unwrap()
        .read_to_string(&mut file)
        .unwrap();
    let toml: RustPackage = toml::from_str(&file).unwrap();
    println!("{:?}", toml);
    create(toml);
}

fn create(info: RustPackage) -> io::Result<()> {
    fs::create_dir_all("target/aur-package")?;
    let mut file = fs::File::create("target/aur-package/PKGBUILD")?;
    println!("");
    // name
    writeln!(file, "pkgname='{}-git'", info.package.name)?;
    // version
    writeln!(file, "pkgver='{}'", info.package.version)?;
    // description
    if let Some(ref desc) = info.package.description {
        writeln!(file, "pkgdesc'{}'", escape(desc))?;
    } else {
        writeln!(
            file,
            "pkgver='{}'",
            user("You do not have a description set, do you want to enter one?")
        )?
    }
    // architectures supported
    writeln!(file, "arch=('all')")?;

    // homepage
    if let Some(ref url) = info.package.homepage {
        writeln!(file, "url='{}", escape(url))?;
    } else {
        writeln!(
            file,
            "url='{}'",
            user("You don't have a homepage set, do you want to enter one?")
        )?
    }

    // license
    if let Some(ref license) = info.package.license {
        writeln!(file, "license=('{}')", escape(license))?;
    } else {
        writeln!(
            file,
            "license=('{}')",
            user("You don't have a license set, do you want to enter one?")
        )?
    }

    // sha sum
    writeln!(file, "sha256sums=('SKIP')")?;

    // makedepends
    writeln!(file, "makedepends=('rust' 'cargo' 'git')")?;
    Ok(())
}

fn escape(s: &str) -> String {
    s.chars().flat_map(char::escape_default).collect()
}

fn user(prompt: &str) -> String {
    Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact()
        .unwrap()
}
