use lib3mf::Model;
use std::collections::HashMap;
use std::fs::File;
use walkdir::WalkDir;

fn main() {
    // Find all test suite directories
    let test_suites_path = "test_suites";
    let mut suites = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(test_suites_path) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let suite_name = entry.file_name().to_string_lossy().to_string();
                // Skip non-suite directories
                if suite_name.starts_with("suite") {
                    let neg_test_path = format!("{}/{}/negative_test_cases", test_suites_path, suite_name);
                    if std::path::Path::new(&neg_test_path).exists() {
                        suites.push(suite_name);
                    }
                }
            }
        }
    }
    
    suites.sort();
    
    let mut failures_by_code: HashMap<String, Vec<(String, String)>> = HashMap::new();
    let mut total_passed = 0;
    let mut total_failed = 0;

    for suite in &suites {
        let path = format!("{}/{}/negative_test_cases", test_suites_path, suite);

        let files: Vec<_> = WalkDir::new(&path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("3mf"))
            .map(|e| e.path().to_path_buf())
            .collect();

        for file_path in files {
            let file_name = file_path.file_name().unwrap().to_str().unwrap();

            // Extract test code from different naming patterns
            let code = if let Some(parts) = file_name.strip_prefix("N_XXX_") {
                let code_part = parts.split('_').next().unwrap_or("XXXX");
                code_part.to_string()
            } else if let Some(parts) = file_name.strip_prefix("N_") {
                // Handle patterns like N_SPX_0802_01.3mf, N_MPX_0401_01.3mf, etc.
                let mut parts_iter = parts.split('_');
                parts_iter.next(); // Skip prefix (SPX, MPX, etc.)
                if let Some(code_part) = parts_iter.next() {
                    code_part.to_string()
                } else {
                    "XXXX".to_string()
                }
            } else {
                "XXXX".to_string()
            };

            match File::open(&file_path) {
                Ok(file) => match Model::from_reader(file) {
                    Ok(_) => {
                        // File was accepted but should have been rejected
                        total_failed += 1;
                        failures_by_code
                            .entry(code.clone())
                            .or_default()
                            .push((format!("{}: {}", suite, file_name), "N/A".to_string()));
                    }
                    Err(_) => {
                        // File was correctly rejected
                        total_passed += 1;
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

    println!("\n=== Detailed Failing Tests by Code ===\n");
    println!("Total Passed (correctly rejected): {}", total_passed);
    println!("Total Failed (incorrectly accepted): {}", total_failed);
    println!("Total Tests: {}", total_passed + total_failed);
    println!(
        "Compliance: {:.1}%\n",
        (total_passed as f64 / (total_passed + total_failed) as f64) * 100.0
    );

    for code in codes {
        let failures = &failures_by_code[code];
        println!("=== Code {} ({} failures) ===", code, failures.len());
        for (i, (file, _err)) in failures.iter().enumerate().take(5) {
            println!("  {}. {}", i + 1, file);
        }
        if failures.len() > 5 {
            println!("  ... and {} more", failures.len() - 5);
        }
        println!();
    }
    
    println!("\n=== Test Suites Analyzed ===");
    for suite in &suites {
        println!("- {}", suite);
    }
}
