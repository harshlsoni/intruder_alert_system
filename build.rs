use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    
    println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo:rustc-link-arg=/MANIFESTINPUT:app.manifest");

    
    let profile = env::var("PROFILE").unwrap();

   
    let target_dir = PathBuf::from(format!("target/{}", profile));

 
    let env_src = Path::new(".env");

    if env_src.exists() {
        let env_dest = target_dir.join(".env");

        fs::create_dir_all(&target_dir).ok();
        fs::copy(env_src, &env_dest).expect("Failed to copy .env");

        println!("cargo:warning=.env copied to {:?}", env_dest);
    } else {
        println!("cargo:warning=.env file not found");
    }

    
    let assets_src = Path::new("assets");
    let assets_dest = target_dir.join("assets");

    if assets_src.exists() {
        copy_dir_all(assets_src, &assets_dest)
            .expect("Failed to copy assets folder");

        println!("cargo:warning=assets copied to {:?}", assets_dest);
    } else {
        println!("cargo:warning=assets folder not found");
    }
}


fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_all(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}