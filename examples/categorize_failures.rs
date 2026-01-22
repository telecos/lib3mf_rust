use lib3mf::Model;
use std::collections::HashMap;
use std::fs::File;
use walkdir::WalkDir;

fn main() {
    let suites = vec![
        ("suite1_core_slice_prod", "negative_test_cases"),
        ("suite2_core_prod_matl", "negative_test_cases"),
        ("suite3_core", "negative_test_cases"),
        ("suite4_core_slice", "negative_test_cases"),
        ("suite5_core_prod", "negative_test_cases"),
        ("suite5_core_prod", "negative_test_cases_prod_alt"),
        ("suite6_core_matl", "negative_test_cases"),
        ("suite7_beam", "negative_test_cases"),
        ("suite8_secure", "negative_test_cases"),
        ("suite9_core_ext", "Negative Tests"),
        ("suite10_boolean", "Negative Tests"),
        ("suite11_Displacement", "Negative Tests"),
    ];

    let mut failures_by_code: HashMap<String, Vec<String>> = HashMap::new();

    for (suite, neg_dir) in suites {
        let path = format!("test_suites/{}/{}", suite, neg_dir);
        if !std::path::Path::new(&path).exists() {
            continue;
        }

        let files: Vec<_> = WalkDir::new(&path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("3mf"))
            .map(|e| e.path().to_path_buf())
            .collect();

        for file_path in files {
            let file_name = file_path.file_name().unwrap().to_str().unwrap();

            // Extract test code (e.g., "0205" from "N_XXX_0205_01.3mf")
            let code = if let Some(parts) = file_name.strip_prefix("N_XXX_") {
                let code_part = parts.split('_').next().unwrap_or("XXXX");
                code_part.to_string()
            } else {
                "XXXX".to_string()
            };

            match File::open(&file_path) {
                Ok(file) => match Model::from_reader(file) {
                    Ok(_) => {
                        // File was accepted but should have been rejected
                        failures_by_code
                            .entry(code)
                            .or_default()
                            .push(format!("{}: {}", suite, file_name));
                    }
                    Err(_) => {
                        // File was correctly rejected - OK
                    }
                },
                Err(e) => {
                    eprintln!("Cannot open {}: {}", file_name, e);
                }
            }
        }
    }

    // Sort and display by code
    let mut codes: Vec<_> = failures_by_code.keys().collect();
    codes.sort();

    println!("\n=== Failing Tests by Code (Category) ===\n");
    for code in codes {
        let failures = &failures_by_code[code];
        println!("Code {} - {} failing tests", code, failures.len());
        for (_i, f) in failures.iter().enumerate().take(3) {
            println!("  - {}", f);
        }
        if failures.len() > 3 {
            println!("  ... and {} more", failures.len() - 3);
        }
        println!();
    }

    println!("\n=== Summary by Code Range ===");
    let mut code_ranges: HashMap<String, usize> = HashMap::new();
    for (code, failures) in &failures_by_code {
        let range = format!("{}XX", &code[..2]);
        *code_ranges.entry(range).or_default() += failures.len();
    }

    let mut ranges: Vec<_> = code_ranges.iter().collect();
    ranges.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

    for (range, count) in ranges {
        println!("{}: {} failures", range, count);
    }
}
