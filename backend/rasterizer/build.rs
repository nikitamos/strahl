use std::{collections::HashSet, path::Path, process::Command};

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  if std::env::var("CARGO_FEATURE_RENDERDOC").is_ok() {
    compile_renderdoc();
  }
  compile_shaders();
}

fn compile_renderdoc() {
  cc::Build::new()
    .std("c11")
    .file("renderdoc.c")
    .compile("amnis-rdoc");
  println!("cargo::rerun-if-changed=renderdoc.c");
  eprintln!()
}

fn compile_shaders() {
  let shaders_dir = Path::new("shaders");
  let dir_print = shaders_dir.display();
  println!("cargo:rerun-if-changed={}/*.slang", dir_print);

  if !check_slang_is_present() {
    println!(
      "cargo:warning=slangc seems to be absent on your system; shader compilation is disabled"
    );
    return;
  }

  let ignored_files: HashSet<&str> = ["globals", "blur"].iter().cloned().collect();

  if !shaders_dir.exists() {
    println!(
      "`{dir_print}` directory not found; current dir is `{:?}`",
      std::env::current_dir()
    );
  }

  let compiled_count = compile_dir(ignored_files, shaders_dir);

  if compiled_count > 0 {
    println!("cargo:warning=Compiled {} shader(s)", compiled_count);
  } else {
    println!("cargo:warning=No .slang files found to compile");
  }
}

fn compile_dir(ignored_files: HashSet<&str>, shaders_dir: &Path) -> i32 {
  let mut compiled_count = 0;
  if let Ok(entries) = std::fs::read_dir(shaders_dir) {
    for entry in entries.filter_map(|e| e.ok()) {
      let path = entry.path();

      if path.extension().and_then(|e| e.to_str()) == Some("slang") {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
          if ignored_files.contains(stem) {
            continue;
          }

          let output_path = shaders_dir.join(format!("{}.wgsl", stem));

          compile_shader(path, output_path);
          compiled_count += 1;
        }
      }
    }
  }
  compiled_count
}

fn compile_shader(path: std::path::PathBuf, output_path: std::path::PathBuf) {
  // println!(
  //   "cargo:warning=Compiling shader: {} -> {}",
  //   path.display(),
  //   output_path.display()
  // );
  match Command::new("slangc")
    .arg(&path)
    .arg("-o")
    .arg(&output_path)
    .status()
  {
    Ok(status) if status.success() => {
      // println!(
      //   "cargo:warning=Successfully compiled: {}",
      //   output_path.display()
      // );
    }
    Ok(status) => {
      panic!(
        "Failed to compile shader '{}'. slangc exited with: {}",
        path.display(),
        status
      );
    }
    Err(e) => {
      panic!(
        "Failed to run slangc for '{}'. Error: {}. Make sure slangc is installed and in PATH.",
        path.display(),
        e
      );
    }
  }
}

fn check_slang_is_present() -> bool {
  match Command::new("slangc")
    .arg("-v")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
    .status()
  {
    Ok(status) => status.success(),
    Err(_) => false,
  }
}
