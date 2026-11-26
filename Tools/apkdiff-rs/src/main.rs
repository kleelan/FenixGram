// Copyright (C) 2026 MeowGram contributors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::ExitCode;
use zip::ZipArchive;

const IGNORED_ENTRIES: [&str; 3] = [
    "META-INF/MANIFEST.MF",
    "META-INF/CERT.RSA",
    "META-INF/CERT.SF",
];

fn is_ignored(name: &str) -> bool {
    IGNORED_ENTRIES.contains(&name)
}

fn compare_entry_bytes(
    first: &mut ZipArchive<File>,
    first_index: usize,
    second: &mut ZipArchive<File>,
    second_index: usize,
) -> zip::result::ZipResult<bool> {
    let mut first_entry = first.by_index(first_index)?;
    let mut second_entry = second.by_index(second_index)?;
    let mut first_buffer = [0_u8; 4096];
    let mut second_buffer = [0_u8; 4096];

    loop {
        let first_read = first_entry.read(&mut first_buffer)?;
        let second_read = second_entry.read(&mut second_buffer)?;
        if first_read != second_read || first_buffer[..first_read] != second_buffer[..second_read] {
            return Ok(false);
        }
        if first_read == 0 {
            return Ok(true);
        }
    }
}

fn compare_apks(first_path: &str, second_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let mut first = ZipArchive::new(File::open(first_path)?)?;
    let mut second = ZipArchive::new(File::open(second_path)?)?;

    let first_indices: Vec<usize> = (0..first.len())
        .filter(|&index| first.by_index(index).map_or(true, |entry| !is_ignored(entry.name())))
        .collect();
    let second_indices: Vec<usize> = (0..second.len())
        .filter(|&index| second.by_index(index).map_or(true, |entry| !is_ignored(entry.name())))
        .collect();

    if first_indices.len() != second_indices.len() {
        println!(
            "APKs has different amount of files ({} != {})",
            first_indices.len(),
            second_indices.len()
        );
        return Ok(false);
    }

    let mut second_by_name: HashMap<String, VecDeque<usize>> = HashMap::new();
    for index in second_indices {
        let name = second.by_index(index)?.name().to_owned();
        second_by_name.entry(name).or_default().push_back(index);
    }

    for first_index in first_indices {
        let name = first.by_index(first_index)?.name().to_owned();
        let Some(second_index) = second_by_name.get_mut(&name).and_then(VecDeque::pop_front) else {
            println!("file {name} not found in second APK");
            return Ok(false);
        };

        if !compare_entry_bytes(&mut first, first_index, &mut second, second_index)? {
            println!("APK file {name} does not match");
            return Ok(false);
        }
    }

    if !second_by_name.is_empty() {
        for (name, indices) in second_by_name {
            for _ in indices {
                println!("file {name} not found in first APK");
            }
        }
        return Ok(false);
    }

    Ok(true)
}

fn run() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: apkdiff <pathToFirstApk> <pathToSecondApk>");
        return ExitCode::from(1);
    }

    let same = if args[1] == args[2] {
        true
    } else {
        match compare_apks(&args[1], &args[2]) {
            Ok(same) => same,
            Err(error) => {
                eprintln!("apkdiff: {error}");
                return ExitCode::from(2);
            }
        }
    };

    if same {
        println!("APKs are the same!");
        ExitCode::SUCCESS
    } else {
        println!("APKs are different!");
        ExitCode::from(1)
    }
}

fn main() {
    std::process::exit(run().code().unwrap_or(1));
}
