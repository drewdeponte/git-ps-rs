use gps as ps;

pub fn show(patch_index: usize) {
    match ps::show(patch_index) {
        Ok(_) => return,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    };
}
