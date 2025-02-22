use std::{env, fs, path::PathBuf};

#[cfg(windows)]
extern crate winres;

fn main() {
	#[cfg(windows)] {
		if std::path::Path::new("icons/icon.ico").exists() {
			let mut res = winres::WindowsResource::new();
			res.set_icon_with_id("icons/icon.ico", "32512");
			res.compile().expect("Unable to find visual studio tools");
		} else {
			panic!("No Icon.ico found. Please add one or check the path");
		}
	}

	let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
	let out = PathBuf::from(format!("{}/../../../", env::var("OUT_DIR").unwrap()));

	#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
	let steam_api_lib = PathBuf::from("win64/steam_api64.dll");
	#[cfg(all(target_os = "windows", target_arch = "x86"))]
	let steam_api_lib = PathBuf::from("win32/steam_api.dll");
	#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
	let steam_api_lib = PathBuf::from("linux64/libsteam_api.so");
	#[cfg(all(target_os = "linux", target_arch = "x86"))]
	let steam_api_lib = PathBuf::from("linux32/libsteam_api.so");
	#[cfg(all(target_os = "macos"))]
	let steam_api_lib = PathBuf::from("osx/libsteam_api.dylib");

	fs::write(out.join("steam_appid.txt"), "4000").unwrap();

	let steam_api_src = src.join("lib/steam_api").join(&steam_api_lib);
	if !steam_api_src.is_file() {
		panic!(format!("\n\nCouldn't find Steam API libraries at {:?}\nSee src-tauri/lib/steam_api/README\n\n", steam_api_src));
	}

	let steam_api_dest = out.join(steam_api_lib.file_name().unwrap());
	fs::copy(steam_api_src, steam_api_dest).unwrap();
}
