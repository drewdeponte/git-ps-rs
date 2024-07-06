use super::utils::print_error_chain;
use gps as ps;

pub fn push(branch_name: String, color: bool) {
    let res = ps::push(branch_name);
    match res {
        Ok(_) => {}
        Err(e) => {
            print_error_chain(color, e.into());
            std::process::exit(1);
        }
    }
}
