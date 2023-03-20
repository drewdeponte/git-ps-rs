use gps as ps;

pub fn sync(patch_index: usize, branch_name: Option<String>) {
    let res = ps::sync(patch_index, branch_name);
    match res {
        Ok(_) => return,
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
