use super::*;

/// Size 自适应：<1MiB 用 KiB，>=1MiB 用 MiB（1 位小数）
#[test]
fn fmt_size_kib_mib_boundary() {
    assert_eq!(fmt_size(0), "0 KiB");
    assert_eq!(fmt_size(1024 * 1024 - 1), "1023 KiB");
    assert_eq!(fmt_size(1024 * 1024), "1.0 MiB");
    assert_eq!(fmt_size(5 * 1024 * 1024 / 2), "2.5 MiB");
}

/// 输出路径：Output/<stem>_rust.usda
#[test]
fn output_path_naming() {
    let (tx, rx) = mpsc::channel();
    let app = App {
        paths: vec![PathBuf::from("Input/My Ship.xml")],
        names: vec!["My Ship.xml".into()],
        selected: 0,
        running: false,
        progress: None,
        tx,
        rx,
        parsed: None,
        pending: None,
        epoch: 0,
        frame: 0,
        cache: HashMap::new(),
        parts: None,
        connections: None,
        materials: None,
        size_in: "…".into(),
        faces: None,
        skel: None,
    };
    assert_eq!(
        app.output_path().unwrap(),
        PathBuf::from("Output/My Ship_rust.usda")
    );
}
