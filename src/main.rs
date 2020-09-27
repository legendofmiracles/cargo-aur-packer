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
    name: String,                  //done
    version: String,               //done
    description: Option<String>,   //done
    authors: Vec<String>,          //done
    keywords: Option<Vec<String>>, //done
    repository: Option<String>,    //done
    homepage: Option<String>,      //done
    license: Option<String>,       //done
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
    // println!("{:?}", toml);
    create(toml).expect("Something went wrong.");
    if Command::new("git")
        .arg("init")
        .arg("target/aur-package")
        .output()
        .is_err()
    {
        println!("Couldn't initialize git repo in target/aur-package");
    }
    std::env::set_current_dir("target/aur-package/").expect("Failed to cd");
    let src_info = Command::new("makepkg").arg("--printsrcinfo").output();
    if src_info.is_ok() {
        let mut file = fs::File::create(".SRCINFO")
            .expect("Failed to create to .SRCINFO file. the pkgbuild was already created tho");
        let output = String::from_utf8(src_info.unwrap().stdout).unwrap();
        writeln!(file, "{}", &output)
            .expect("Failed to write to .SRCINFO file. the pkgbuild was already created tho");
    }
    Command::new("git")
        .arg("add")
        .arg("--all")
        .output()
        .expect("Failed to add all files to git");

    Command::new("git")
        .arg("commit")
        .arg("-am")
        .arg("First commit with all the files")
        .output()
        .expect("Failed to commit git");
}

fn create(info: RustPackage) -> io::Result<()> {
    fs::create_dir_all("target/aur-package")?;
    let mut file = fs::File::create("target/aur-package/PKGBUILD")?;
    println!("");
    writeln!(file, "# PKGBUILD created by using cargo-aur")?;
    // name
    writeln!(file, "pkgname='{}-git'", info.package.name)?;
    // version
    writeln!(file, "pkgver={}", info.package.version)?;
    // pkgrel
    writeln!(file, "pkgrel=1")?;
    // description
    if let Some(ref desc) = info.package.description {
        writeln!(file, "pkgdesc='{}'", escape(desc))?;
    } else {
        writeln!(
            file,
            "pkgdesc='{}'",
            user("You do not have a description set, do you want to enter one?")
        )?
    }
    // architectures supported
    writeln!(file, "arch=('any')")?;

    // homepage
    if let Some(ref url) = info.package.homepage {
        writeln!(file, "url='{}'", escape(url))?;
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

    // source
    if let Some(ref src) = info.package.repository {
        writeln!(
            file,
            "source=('{}-git::git+{}')",
            info.package.name,
            escape(src)
        )?;
    } else {
        writeln!(
            file,
            "source=('{}-git::git+{}')",
            info.package.name,
            user("You don't have a repository set, do you want to enter one?")
        )?
    }

    // build functions
    writeln!(
        file,
        r#"
pkgver() {{
 cd "$pkgname"
 echo "$(grep '^version =' Cargo.toml|head -n1|cut -d\" -f2).$(git rev-list --count HEAD).g$(git rev-parse --short HEAD)" | tr '-' '.'
}}

build() {{
   cd "$pkgname"
   cargo build --release --locked --all-features --target-dir=target
}}

check() {{
  cd "$pkgname"
  cargo test --release --locked --target-dir=target
}}

package() {{
  cd "$pkgname"
  install -Dm 755 target/release/{name} -t "${{pkgdir}}/usr/bin"
  # install -Dm 755 $pkgname/LICENSE "${{pkgdir}}/usr/share/licenses/{name}"
}}
"#,
        name = info.package.name
    )?;
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
