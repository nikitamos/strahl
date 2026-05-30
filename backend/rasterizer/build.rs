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
    .std("c++17")
    .file("renderdoc.cc")
    .compile("amnis-rdoc");
  println!("cargo::rerun-if-changed=renderdoc.cc");
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

  let ignored_files: HashSet<&str> = ["globals"].iter().cloned().collect();

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

      if path.extension().and_then(|e| e.to_str()) == Some("slang")
        && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
      {
        if ignored_files.contains(stem) {
          continue;
        }

        let output_path = shaders_dir.join(format!("{}.wgsl", stem));

        compile_shader(path.clone(), output_path.clone());

        // Post-process specific shaders to apply Makefile-style sed replacements
        post_process_shader(stem, &output_path);

        compiled_count += 1;
      }
    }
  }
  compiled_count
}

fn compile_shader(path: std::path::PathBuf, output_path: std::path::PathBuf) {
  match Command::new("slangc")
    .arg(&path)
    .arg("-o")
    .arg(&output_path)
    .status()
  {
    Ok(status) if status.success() => {}
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

fn post_process_shader(stem: &str, output_path: &Path) {
  match stem {
    "raster-pipeline" => {
      replace_line_containing(
        output_path,
        "var<uniform> body_0 : Body_std430_0;",
        "var<immediate> body_0 : Body_std430_0;",
      );
    }
    "blur" => {
      replace_line_containing(
        output_path,
        "var<uniform> push_0 : PushConstants_std430_0;",
        "var<immediate> push_0 : PushConstants_std430_0;",
      );
    }
    _ => {}
  }
}

fn replace_line_containing(path: &Path, pattern: &str, replacement: &str) {
  let content = std::fs::read_to_string(path)
    .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

  let has_trailing_newline = content.ends_with('\n');

  let new_content = content
    .lines()
    .map(|line| {
      if line.contains(pattern) {
        replacement.to_string()
      } else {
        line.to_string()
      }
    })
    .collect::<Vec<_>>()
    .join("\n");

  // Preserve original trailing newline behavior
  let new_content = if has_trailing_newline && !new_content.ends_with('\n') {
    format!("{}\n", new_content)
  } else {
    new_content
  };

  std::fs::write(path, new_content)
    .unwrap_or_else(|e| panic!("Failed to write {}: {}", path.display(), e));
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
