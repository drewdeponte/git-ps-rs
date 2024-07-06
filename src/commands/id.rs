use super::utils::print_error_chain;
use gps as ps;

pub fn id(color: bool) {
    let res = ps::id();
    match res {
        Ok(_) => {}
        Err(e) => {
            print_error_chain(color, e.into());
            std::process::exit(1);
        }
    }
}
