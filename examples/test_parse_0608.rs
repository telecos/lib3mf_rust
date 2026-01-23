use lib3mf::Model;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("test_suites/suite2_core_prod_matl/negative_test_cases/N_XPM_0608_01.3mf")?;
    let model = Model::from_reader(file)?;
    
    println!("Color groups:");
    for cg in &model.resources.color_groups {
        println!("  ColorGroup {}: {} colors", cg.id, cg.colors.len());
        for (i, color) in cg.colors.iter().enumerate() {
            println!("    Color {}: {:?}", i, color);
        }
    }
    
    println!("\nObjects:");
    for obj in &model.resources.objects {
        println!("  Object {}: pid={:?}, pindex={:?}", obj.id, obj.pid, obj.pindex);
    }
    
    Ok(())
}
