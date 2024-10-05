use memmap2::Mmap;
use std::env;
use std::error::Error;
use std::fs::{copy, create_dir_all, read_dir, remove_file, remove_dir_all, rename, File};
use std::io::Write;
use std::path::Path;
use std::process::exit;
use std::result::Result;
use clap::Parser;

const DIST_DIR: &str = env!("PLAYFAIR_DIST_DIR");

const SURROGATE_JS: &str = r#"import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

let binding;

const file = './{{NAME}}_orig.js';

if (!fs.existsSync(path.join(__dirname, file))) {
    binding = (await import('./.package/package.mjs')).default;
} else {
    binding = await import(file);
}

export default binding.default;
"#;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
enum Args {
    #[command(name = "pack", about = "Protect a file with Playfair")]
    Pack {
        #[arg(required = true, help = "the file to protect")]
        file: String,

        #[arg(required = true, help = "the key to encrypt the file with")]
        key: String,

        #[arg(required = true, help = "the URL where to send requests at runtime to retrieve the decryption key")]
        url: String,
    },

    #[command(name = "strip", about = "Remove the Playfair protection for a file (requires original _orig file)")]
    Strip {
        #[arg(required = true, help = "the file to remove the protection for (normal filename, not _orig)")]
        file: String
    },

    #[command(name = "recover", about = "Recover a file protected by Playfair (doesn't require original _orig file)")]
    Restore {
        #[arg(required = true, help = "the file to restore (normal filename, not _orig)")]
        file: String,

        #[arg(required = false, help = "the decryption key; if you don't have it, Playfair will fetch it from the stored URL")]
        key: Option<String>
    },
}

fn copy_playfair_dist(src: &Path, dest: &Path) -> Result<(), Box<dyn Error>> {
    for entry in read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if dest_path.exists() {
            println!(
                "Skipping file {} since it already exists",
                dest_path.file_name().unwrap().to_string_lossy()
            );
        } else {
            copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args {
        Args::Pack {
            file,
            key,
            url,
        } => {
            pack(&file, &key, &url)
        }
        Args::Strip {
            file
        } => {
            strip(&file)
        }
        Args::Restore {
            file,
            key
        } => {
            recover(&file, key)
        }
    }
}

fn pack(file: &String, key: &String, url: &String) -> Result<(), Box<dyn Error>> {
    let path = Path::new(file);

    if !path.exists() {
        println!("File {} does not exist", file);
        exit(2);
    }

    if path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .ends_with("_orig")
    {
        println!(
            "Filename ends with '_orig', which means it might've been generated by Playfair"
        );
        println!("If you know you haven't used Playfair, and this is actually a normal file, feel free to proceed");

        let mut answer: String = String::new();

        loop {
            println!("Proceed? (Y/N) ");

            if std::io::stdin().read_line(&mut answer).is_ok() {
                if answer.starts_with('y') || answer.starts_with('Y') {
                    break;
                } else if answer.starts_with('n') || answer.starts_with('N') {
                    exit(0);
                }
            } else {
                println!("stdin is closed, assuming YES");
                break;
            }

            answer.clear();
        }
    }

    println!("Creating .package directory");
    {
        create_dir_all(".package")?;
    }

    let pkg = Path::new(path.parent().unwrap()).join(".package");

    println!("Copying Playfair into .package from {}", DIST_DIR);
    {
        let dist = Path::new(DIST_DIR);

        if !dist.exists() {
            println!("Cannot find {}", dist.display());
            println!("If the Playfair repo dir moved, please re-compile and install playfair-cli from the new location");
            exit(3);
        }
        copy_playfair_dist(Path::new(DIST_DIR), &pkg)?;
    }

    let original = format!(
        "{}_orig.{}",
        path.file_stem().unwrap().to_string_lossy(),
        path.extension().unwrap().to_string_lossy()
    );

    if path.parent().unwrap().join(&original).exists() {
        println!("File {} already exists", &original);
        println!("This means that Playfair was already used on {}", path.file_name().unwrap().to_string_lossy());
        println!("Please remove the Playfair protection via the 'strip' command, and try again");
        println!("If you haven't used Playfair, and {} is actually a normal file, please rename it to something else", &original);
        exit(4);
    }

    println!(
        "Renaming original file {} -> {}",
        path.file_name().unwrap().to_string_lossy(),
        original
    );
    {
        rename(file, &original)?;
    }

    println!(
        "Creating surrogate {} file",
        path.file_name().unwrap().to_string_lossy()
    );
    {
        let surrogate_js =
            SURROGATE_JS.replace("{{NAME}}", &path.file_stem().unwrap().to_string_lossy());

        let mut surrogate = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        surrogate.write_all(surrogate_js.as_bytes())?;
    }

    let out_path = pkg.join("package.dat");

    println!("Packing file to {}", out_path.to_string_lossy());
    {
        let f = File::open(&original)?;
        let map = unsafe { Mmap::map(&f)? };

        playfair_core::pack(&map, key, url, &out_path.to_string_lossy())?;
    }

    println!("Packed successfully!");

    Ok(())
}

fn strip(file: &String) -> Result<(), Box<dyn Error>> {
    let path = Path::new(file);

    if !path.exists() {
        println!("File {} does not exist", file);
        exit(5);
    }

    let original = format!(
        "{}_orig.{}",
        path.file_stem().unwrap().to_string_lossy(),
        path.extension().unwrap().to_string_lossy()
    );
    let original_path = path.with_file_name(&original);

    if !original_path.exists() {
        println!("Original file {} does not exist", &original);
        println!("If you don't have access to it, please use the 'recover' command instead");
        exit(6);
    }

    println!("Removing .package dir");
    {
        remove_dir_all(Path::new(path.parent().unwrap()).join(".package"))?;
    }

    println!("Removing surrogate file {}", path.file_name().unwrap().to_string_lossy());
    {
        remove_file(path)?;
    }

    println!("Renaming original file {} -> {}", original, path.file_name().unwrap().to_string_lossy());
    {
        rename(original_path, path)?;
    }

    println!("Stripped successfully!");

    Ok(())
}

fn recover(file: &String, key: Option<String>) -> Result<(), Box<dyn Error>> {
    let path = Path::new(file);

    let original = format!(
        "{}_orig.{}",
        path.file_stem().unwrap().to_string_lossy(),
        path.extension().unwrap().to_string_lossy()
    );
    let original_path = path.with_file_name(&original);

    if original_path.exists() {
        println!("Original file {} already exists", original);
        println!("If this is the correct file and you want to simply remove the protection, use the 'strip' command");
        println!("If you want to forcefully recover from the protected file, please delete {}", original);
        exit(7);
    }

    let pkg = Path::new(path.parent().unwrap()).join(".package");
    let in_path = pkg.join("package.dat");

    if !in_path.exists() {
        println!("File {} does not exist", in_path.display());
        println!("Make sure you are running this command in the original dir that has the subdir '.package'");
        exit(8);
    }

    let in_file = File::open(in_path)?;

    let map = unsafe { Mmap::map(&in_file)? };

    let out = playfair_core::unpack(&map, key.as_ref());

    if out.is_err() {
        println!("Failed to recover file; the key is wrong, or the data is corrupted");
        println!("If you don't know the key, don't pass it as an argument; let Playfair retrieve it from the stored URL");
        exit(9);
    }

    let out = out.unwrap();

    println!("Writing data to {}", file);

    let mut out_file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

    out_file.write_all(&out)?;

    println!("Recovered successfully!");

    Ok(())
}