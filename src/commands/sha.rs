use super::utils::print_error_chain;
use gps as ps;

pub fn sha(patch_index: usize, color: bool, exclude_newline: bool) {
    let res = ps::sha::sha(patch_index, exclude_newline);
    match res {
        Ok(_) => {}
        Err(e) => {
            print_error_chain(color, e.into());
            std::process::exit(1);
        }
    }
}
