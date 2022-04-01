use super::git;
use super::ps;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct RequestReviewRecord {
    patch_stack_id: Uuid,
    branch_name: String,
    commit_id: String,
    published: Option<bool>,
    location_agnostic_hash: Option<String>
}

pub fn ls() {
    let repo = git::create_cwd_repo().unwrap();

    // let path_str = format!("{}{}", repo.path().to_str().unwrap(), "patch-stack-review-requests.json");
    // let path = Path::new(&path_str);
    // let display = path.display();

    // let mut file = match File::open(&path) {
    //     Err(why) => panic!("couldn't open {}: {}", display, why),
    //     Ok(file) => file,
    // };

    // let mut s = String::new();
    // match file.read_to_string(&mut s) {
    //     Err(why) => panic!("couldn't read {}: {}", display, why),
    //     Ok(_) => print!("{} contains:\n{}", display, s),
    // }

    // let rr_records: Vec<RequestReviewRecord> = serde_json::from_str(s.as_str()).unwrap();
    // println!("deserialized = {:?}", rr_records);

    let patch_stack = ps::get_patch_stack(&repo).unwrap();
    let list_of_patches = ps::get_patch_list(&repo, patch_stack);

    for patch in list_of_patches.into_iter().rev() {
        println!("{}     {} - {}", patch.index, patch.oid, patch.summary)
    }
}
