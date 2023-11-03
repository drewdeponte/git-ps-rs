use gps as ps;

pub fn sha(patch_index: usize) {
    let res = ps::sha::sha(patch_index);
    match res {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
