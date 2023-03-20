use gps as ps;

pub fn batch_request_review(patch_indexes: Vec<usize>, color: bool) {
    match ps::batch_request_review(patch_indexes, color) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
