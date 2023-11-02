use gps as ps;


pub fn amend(patch_index: usize) {
    let res = ps::amend(patch_index);
    match res {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
