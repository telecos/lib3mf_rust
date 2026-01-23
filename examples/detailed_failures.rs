use lib3mf::Model;
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

    let mut failures = Vec::new();

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

            match File::open(&file_path) {
                Ok(file) => match Model::from_reader(file) {
                    Ok(_) => {
                        // File was accepted but should have been rejected
                        failures.push(format!("{:30} {}", suite, file_name));
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

    println!("\n=== All Failed Tests (Incorrectly Accepted) ===\n");
    for f in failures {
        println!("{}", f);
    }
}
