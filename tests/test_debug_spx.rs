#[cfg(test)]
mod test {
    use lib3mf::{Model, ParserConfig};
    use std::fs::File;

    #[test]
    fn debug_n_spx_0417() {
        let config = ParserConfig::with_all_extensions();
        let file = File::open("test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_0417_01.3mf").unwrap();
        match Model::from_reader_with_config(file, config) {
            Ok(model) => {
                println!("✗ PARSED SUCCESSFULLY (should have failed!)");
                for ss in &model.resources.slice_stacks {
                    println!("SliceStack {}: {} slices", ss.id, ss.slices.len());
                    for (i, slice) in ss.slices.iter().enumerate() {
                        if !slice.polygons.is_empty() && slice.vertices.is_empty() {
                            println!("  Slice {} (ztop={}): {} polygons, {} vertices - SHOULD FAIL!",
                                i, slice.ztop, slice.polygons.len(), slice.vertices.len());
                        }
                    }
                }
            },
            Err(e) => println!("✓ Failed as expected: {}", e),
        }
    }

    #[test]
    fn debug_n_spx_0419() {
        let config = ParserConfig::with_all_extensions();
        let file = File::open("test_suites/suite1_core_slice_prod/negative_test_cases/N_SPX_0419_01.3mf").unwrap();
        match Model::from_reader_with_config(file, config) {
            Ok(model) => {
                println!("✗ PARSED SUCCESSFULLY (should have failed!)");
                println!("Thumbnail: {:?}", model.thumbnail);
            },
            Err(e) => println!("✓ Failed as expected: {}", e),
        }
    }
}
