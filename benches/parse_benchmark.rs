use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use lib3mf::Model;
use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// Generate a 3MF file with a specified number of vertices and triangles
fn generate_3mf(vertices: usize, triangles: usize) -> NamedTempFile {
    let temp_file = NamedTempFile::new().unwrap();
    let mut zip = ZipWriter::new(temp_file.reopen().unwrap());
    let options = SimpleFileOptions::default();

    // Create [Content_Types].xml
    let content_types = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml"/>
</Types>"#;

    zip.start_file("[Content_Types].xml", options).unwrap();
    zip.write_all(content_types.as_bytes()).unwrap();

    // Create _rels/.rels
    let rels = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rel0" Target="/3D/3dmodel.model" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/>
</Relationships>"#;

    zip.start_file("_rels/.rels", options).unwrap();
    zip.write_all(rels.as_bytes()).unwrap();

    // Generate 3dmodel.model with many vertices and triangles
    let mut model_xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="millimeter" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02">
    <resources>
        <object id="1" type="model">
            <mesh>
                <vertices>
"#,
    );

    // Generate vertices in a grid pattern
    for i in 0..vertices {
        let x = (i % 100) as f64;
        let y = (i / 100) as f64;
        let z = 0.0;
        model_xml.push_str(&format!(
            "                    <vertex x=\"{}\" y=\"{}\" z=\"{}\"/>\n",
            x, y, z
        ));
    }

    model_xml.push_str(
        r#"                </vertices>
                <triangles>
"#,
    );

    // Generate triangles with valid topology
    // Each triangle uses 3 consecutive vertices when possible
    // For excess triangles beyond vertices/3, we reuse vertices in a valid way
    for i in 0..triangles {
        let base = (i * 3) % (vertices.saturating_sub(2));
        let v1 = base;
        let v2 = base + 1;
        let v3 = base + 2;
        model_xml.push_str(&format!(
            "                    <triangle v1=\"{}\" v2=\"{}\" v3=\"{}\"/>\n",
            v1, v2, v3
        ));
    }

    model_xml.push_str(
        r#"                </triangles>
            </mesh>
        </object>
    </resources>
    <build>
        <item objectid="1"/>
    </build>
</model>"#,
    );

    zip.start_file("3D/3dmodel.model", options).unwrap();
    zip.write_all(model_xml.as_bytes()).unwrap();

    zip.finish().unwrap();
    temp_file
}

fn bench_parse_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_small");

    for &(vertices, triangles) in &[(100, 50), (500, 250), (1000, 500)] {
        let temp_file = generate_3mf(vertices, triangles);
        let path = temp_file.path();

        group.bench_with_input(
            BenchmarkId::new(
                "vertices_triangles",
                format!("{}v_{}t", vertices, triangles),
            ),
            &path,
            |b, &path| {
                b.iter(|| {
                    let file = File::open(path).unwrap();
                    black_box(Model::from_reader(file).unwrap())
                });
            },
        );
    }

    group.finish();
}

fn bench_parse_medium(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_medium");

    for &(vertices, triangles) in &[(5000, 2500), (10000, 5000)] {
        let temp_file = generate_3mf(vertices, triangles);
        let path = temp_file.path();

        group.bench_with_input(
            BenchmarkId::new(
                "vertices_triangles",
                format!("{}v_{}t", vertices, triangles),
            ),
            &path,
            |b, &path| {
                b.iter(|| {
                    let file = File::open(path).unwrap();
                    black_box(Model::from_reader(file).unwrap())
                });
            },
        );
    }

    group.finish();
}

fn bench_parse_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_large");
    group.sample_size(10); // Reduce sample size for large files

    for &(vertices, triangles) in &[(50000, 25000), (100000, 50000)] {
        let temp_file = generate_3mf(vertices, triangles);
        let path = temp_file.path();

        group.bench_with_input(
            BenchmarkId::new(
                "vertices_triangles",
                format!("{}v_{}t", vertices, triangles),
            ),
            &path,
            |b, &path| {
                b.iter(|| {
                    let file = File::open(path).unwrap();
                    black_box(Model::from_reader(file).unwrap())
                });
            },
        );
    }

    group.finish();
}

fn bench_parse_real_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_real_files");

    // Test with real files if they exist
    let test_files = ["test_files/core/box.3mf", "test_files/core/torus.3mf"];

    for file_path in &test_files {
        if std::path::Path::new(file_path).exists() {
            let file_name = std::path::Path::new(file_path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();

            group.bench_function(file_name, |b| {
                b.iter(|| {
                    let file = File::open(file_path).unwrap();
                    black_box(Model::from_reader(file).unwrap())
                });
            });
        }
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_small,
    bench_parse_medium,
    bench_parse_large,
    bench_parse_real_files
);
criterion_main!(benches);
