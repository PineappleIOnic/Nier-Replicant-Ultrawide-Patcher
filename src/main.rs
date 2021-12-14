use dialoguer::console::Term;
use dialoguer::{theme::ColorfulTheme, Select};
use dirs;
use hex_literal::{self, hex};
use spinners;
use spinners::{Spinner, Spinners};
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone)]
struct EngineRatio {
    name: String,
    hex: [u8; 4],
    height: f32,
    width: f32,
}

fn main() {
    println!("Nier Replicant Ultrawide Patcher 1.0");

    let game_path = match detect_game_location() {
        Ok(path) => path,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    println!("Found game at {:?}!", &game_path);

    backup(&game_path);

    let ratio = match ratio_select() {
        Ok(ratio) => ratio,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    println!("Selected ratio: {}", ratio.name);

    patch_aspect_ratio(&game_path, &ratio);

    correct_position(&game_path, &ratio);

    fix_ui_scaling(&game_path, &ratio);
}

fn fix_ui_scaling(game_path: &PathBuf, ratio: &EngineRatio) {
    println!("This task downloads files from github they can be found here: todo");
    let sp = Spinner::new(&Spinners::Dots9, "Fixing UI Scaling".into());

    let source = "https://raw.githubusercontent.com/PineappleIOnic/Nier-Replicant-Ultrawide-Patcher/main/UIPatch";

    let filesToDownload = vec![
        "/d3dx.ini",
        "/d3dcompiler_46.dll",
        "/d3d11.dll",
        "/ShaderFixes/0a2c2125f4a421a5-vs_replace.txt",
        "/ShaderFixes/3dvision2sbs_sli_downscale_pass1.hlsl",
        "/ShaderFixes/3dvision2sbs_sli_downscale_pass2.hlsl",
        "/ShaderFixes/3dvision2sbs.hlsl",
        "/ShaderFixes/3dvision2sbs.ini",
        "/ShaderFixes/dc88834b3469cba8-vs_replace.txt",
        "/ShaderFixes/mouse.hlsl",
        "/ShaderFixes/mouse.ini",
        "/ShaderFixes/upscale.hlsl",
        "/ShaderFixes/upscale.ini",
    ];

    // Create ShaderFixes folder if it doesn't already exist.
    let shader_fixes_path = game_path.clone().join("ShaderFixes");
    if !shader_fixes_path.exists() {
        std::fs::create_dir_all(&shader_fixes_path).unwrap();
    }


    // Start downloading files needed.
    for file in filesToDownload {
        match ureq::get(&format!("{}{}", source, file)).call() {
            Ok(res) => {
                let len = res
                    .header("Content-Length")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap();

                let mut bytes: Vec<u8> = Vec::with_capacity(len);
                match res.into_reader().take(10_000_000).read_to_end(&mut bytes) {
                    Ok(_) => (),
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                let mut file_path = game_path.clone();
                file_path.push(file);

                let mut file = match std::fs::File::create(&file_path) {
                    Ok(file) => file,
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                match file.write_all(&bytes) {
                    Ok(_) => {
                        println!("Downloaded {}", file_path.to_str().unwrap());
                    }
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                }
            }
            Err(err) => {
                println!("{}", err);
                return;
            }
        };
    }
}

fn patch_aspect_ratio(game_dir_path: &PathBuf, ratio: &EngineRatio) {
    let sp = Spinner::new(&Spinners::Dots9, "Patching Aspect Ratio".into());

    let mut game_path = game_dir_path.clone();
    game_path.push("NieR Replicant ver.1.22474487139.exe");

    // Load game executable into a buffer
    let mut game_exe_file = std::fs::File::open(&game_path).unwrap();

    // Update all instances of old game's ratio
    let original_ratio = hex!("39 8E E3 3F");

    let new_ratio = ratio.hex;

    let mut buffer: Vec<u8> = Vec::new();
    let length = game_exe_file.read_to_end(&mut buffer).unwrap();

    for i in 0..length {
        if (i + 4) > length {
            break;
        }
        if buffer[i..i + 4] == original_ratio {
            buffer[i..i + 4].clone_from_slice(&new_ratio);
        }
    }

    // Write patched executable to disk
    let mut patched_exec_file = std::fs::File::create(&game_path).unwrap();

    patched_exec_file.write_all(&buffer).unwrap();

    sp.stop();
    println!(" Done!");
}

fn correct_position(game_dir_path: &PathBuf, ratio: &EngineRatio) {
    let sp = Spinner::new(&Spinners::Dots9, "Removing Black Bars".into());

    let mut game_path = game_dir_path.clone();
    game_path.push("NieR Replicant ver.1.22474487139.exe");

    // Load game executable into a buffer
    let mut game_exe_file = std::fs::File::open(&game_path).unwrap();

    // Update all instances of old game's ratio
    let original_ratio = hex!("00 00 10 41 00 00 50 41 00 00 80 41 00 00 00 00");

    let mut new_ratio = original_ratio.clone();
    new_ratio[0..4].clone_from_slice(&ratio.width.to_le_bytes());
    new_ratio[8..12].clone_from_slice(&ratio.height.to_le_bytes());

    let mut buffer: Vec<u8> = Vec::new();
    let length = game_exe_file.read_to_end(&mut buffer).unwrap();

    for i in 0..length {
        if (i + 16) > length {
            break;
        }
        if buffer[i..i + 16] == original_ratio {
            buffer[i..i + 16].clone_from_slice(&new_ratio);
        }
    }

    // Write patched executable to disk
    let mut patched_exec_file = std::fs::File::create(&game_path).unwrap();

    patched_exec_file.write_all(&buffer).unwrap();

    sp.stop();
    println!(" Done!");
}

fn ratio_select() -> Result<EngineRatio, std::io::Error> {
    println!("Select your display ratio:");

    let common_ratios = vec![
        EngineRatio {
            name: "21:9 (2560x1080)".into(),
            hex: hex!("26 B4 17 40"),
            height: 21.0,
            width: 9.0,
        },
        EngineRatio {
            name: "21:9 (3440x1440)".into(),
            hex: hex!("8E E3 18 40"),
            height: 21.0,
            width: 9.0,
        },
        EngineRatio {
            name: "21:9 (3840x1600)".into(),
            hex: hex!("9A 99 19 40"),
            height: 21.0,
            width: 9.0,
        },
        EngineRatio {
            name: "32:10".into(),
            hex: hex!("CD CC 4C 40"),
            height: 32.0,
            width: 10.0,
        },
        EngineRatio {
            name: "32:9".into(),
            hex: hex!("39 8E 63 40"),
            height: 32.0,
            width: 9.0,
        },
    ];

    // Create array with only the names
    let mut names = Vec::new();
    for ratio in common_ratios.clone() {
        names.push(ratio.name);
    }

    let selection;

    loop {
        let buffer = Select::with_theme(&ColorfulTheme::default())
            .items(&names)
            .default(0)
            .interact_on_opt(&Term::stderr())?;

        if buffer.is_some() {
            selection = buffer.unwrap();
            break;
        }

        println!("Please select a ratio.");
    }

    Ok(common_ratios[selection].clone())
}

fn backup(game_path: &PathBuf) {
    let mut backup_path = game_path.clone();
    backup_path.push("NieR Replicant ver.1.22474487139.exe.bak");

    let mut game_path = game_path.clone();
    game_path.push("NieR Replicant ver.1.22474487139.exe");

    if backup_path.exists() {
        println!("Restore original game backup for patching? (highly recommended) [y/n]");

        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();

        if buffer.trim() == "y" {
            std::fs::copy(&backup_path, &game_path).unwrap();
            println!("Successfully restored backup.");
            return;
        }
    }

    println!("Create a backup of the game? (y/n)");
    let mut backup = String::new();
    std::io::stdin().read_line(&mut backup).unwrap();
    if backup.trim() == "y" {
        println!("Creating backup...");

        std::fs::copy(game_path, &backup_path).unwrap();
        println!("Backup created at {:?}!", &backup_path);
    }
}

fn detect_game_location() -> Result<PathBuf, String> {
    // Check if it's in the current directory
    if Path::new("NieR Replicant ver.1.22474487139.exe").exists() {
        return Ok(PathBuf::from("./"));
    }

    // Go through common directories

    // Steam Windows Directory
    if Path::new("C:/Program Files (x86)/Steam/steamapps/common/NieR Replicant ver.1.22474487139/NieR Replicant ver.1.22474487139.exe").exists() {
        return Ok(PathBuf::from("C:/Program Files (x86)/Steam/steamapps/common/NieR Replicant ver.1.22474487139/"));
    }

    // Steam Mac OS Directory (not sure who is playing this game on MacOS but oh well)
    if Path::new("~/Library/Application Support/Steam/steamapps/common/NieR Replicant ver.1.22474487139/NieR Replicant ver.1.22474487139.exe").exists() {
        return Ok(PathBuf::from("/Applications/Steam/steamapps/common/NieR Replicant ver.1.22474487139/"));
    }

    let home_dir = dirs::home_dir().unwrap();

    // Steam Linux Directory
    if Path::new(&format!("{}/.steam/steam/steamapps/common/NieR Replicant ver.1.22474487139/NieR Replicant ver.1.22474487139.exe", home_dir.to_str().unwrap())).exists() {
        return Ok(PathBuf::from(format!("{}/.steam/steam/steamapps/common/NieR Replicant ver.1.22474487139/", home_dir.to_str().unwrap())));
    }

    // Another possible linux directory
    if Path::new(&format!("{}/.local/share/Steam/steamapps/common/NieR Replicant ver.1.22474487139/NieR Replicant ver.1.22474487139.exe", home_dir.to_str().unwrap())).exists() {
        return Ok(PathBuf::from(format!("{}/.local/share/Steam/steamapps/common/NieR Replicant ver.1.22474487139/", home_dir.to_str().unwrap())));
    }

    // Could not find the game executable automatically. Ask the user.
    println!("Could not find the game executable automatically. Please enter the path manually:");
    let mut path = String::new();
    std::io::stdin()
        .read_line(&mut path)
        .expect("Failed to read line");
    path = path.trim().to_string();

    if Path::new(&path).exists() {
        return Ok(PathBuf::from(path));
    }

    Err("Could not find the game executable".to_string())
}
