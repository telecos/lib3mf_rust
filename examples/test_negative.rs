use lib3mf::Model;
use std::fs::File;

fn main() {
    let files = vec![
        "test_suites/suite3_core/negative_test_cases/N_XXX_0202_01.3mf",
        "test_suites/suite3_core/negative_test_cases/N_XXX_0203_01.3mf",
        "test_suites/suite3_core/negative_test_cases/N_XXX_0204_01.3mf",
        "test_suites/suite3_core/negative_test_cases/N_XXX_0405_01.3mf",
        "test_suites/suite3_core/negative_test_cases/N_XXX_0503_01.3mf",
    ];

    for file_path in files {
        match File::open(file_path) {
            Ok(file) => match Model::from_reader(file) {
                Ok(_) => println!("❌ {} - ACCEPTED (should be rejected)", file_path),
                Err(e) => println!("✅ {} - REJECTED: {}", file_path, e),
            },
            Err(e) => println!("⚠️  {} - Cannot open: {}", file_path, e),
        }
    }
}
