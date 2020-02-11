use git2::{Repository, BranchType, Branch, RepositoryOpenFlags};
use chrono::{Utc, TimeZone, DateTime};
use std::option::Option as Option;
use colored::*;
use std::env;
use std::process::exit;
use std::ffi::OsStr;

struct BranchCommitTime {
    branch_name: String,
    last_commit: DateTime<Utc>,
    hash: String
}

fn main() {

    // to allow piping the result to other programs
    //https://github.com/rust-lang/rust/issues/46016
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            println!("{}","Using current dir as git repository to analyse.".green());
            show_branches_and_time(env::current_dir().unwrap().to_str().unwrap());
        },
        2 => {
            println!("{}{}{}", "Using ".green() ,args[1].green(), " as git repository to analyse.".green());
            show_branches_and_time(args[1].as_str());
        },
        _ => {
            println!("{}", "Unexpected number of arguments".red());
            exit(1);
        }
    };

}

fn show_branches_and_time(dir: &str) {
//    let repo = match Repository::open("/Users/jos/dev/git/cer/arachne-backend") {
//    let repo = match Repository::open(dir) {
    let flags = RepositoryOpenFlags::empty();
    let repo = match Repository::open_ext(dir, flags, &[] as &[&OsStr]) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to init: {}", e),
    };

    // get all the local branches, and for each get the name and last commit time
    // and return them as a new list of BranchCommitTime structs
    let branches_list = get_all_local_branches(&repo);
    let bct_list = branches_list.filter_map(|branch| {

        // first get the name, or ignore the field when the name can't be determined
        return match branch {
            Ok((branch, _)) => {
                let branch_name = get_branch_name(&branch);
                let last_commit = get_branch_last_commit(&branch, &repo);

                // flatmap these options to create an Option<BranchCommitTime>
                let bct = branch_name
                    .and_then(|n| last_commit.map(|t|
                        BranchCommitTime { branch_name: n, last_commit: t.1, hash: t.0 }
                    ));

                bct
            }
            Err(_) => None
        };
    });

    // collect the iterator in a vector so we can sort it. This has to
    // be a mutable one, since we do sorting in place. Finally print out
    // the sorted list to console.
    let mut bct_v: Vec<_> = bct_list.collect();
    bct_v.sort_by(|a, b| b.last_commit.cmp(&a.last_commit));
    bct_v.iter().for_each(|bc| {
        // for article purposes we'll hash the names
        // let hashed_name = hash_string(bc.branch_name.as_str());
        println!("{} : {} : {}", bc.last_commit.to_string().yellow(), bc.hash, bc.branch_name.blue());
        }
    );
}

/// replace all the characters in a string, except the first 3
//fn hash_string(name: &str) -> String {
//    let mut hashed_name = String::from("");
//    for (pos, e) in name.chars().enumerate() {
//        match pos {
//            0..=2 => hashed_name.push(e),
//            _ => hashed_name.push('#')
//        };
//    }
//    return hashed_name
//}

/// We get the name, and convert it to a string, so we don't
/// run into ownership issues, or need to pass the lifetime around.
fn get_branch_name(branch: &Branch) -> Option<String> {
    let name = match branch.name() {
        Ok(r) => r.map(|n| String::from(n)),
        Err(_) => None
    };

    return name;
}

/// get the last commit time from a branch, if it fails return none
fn get_branch_last_commit(branch: &Branch, repo: &Repository) -> Option<(String,DateTime<Utc>)> {
    let p = branch.get().target().and_then(|oid| {
        let commit = repo.find_commit(oid);

        let t = commit.map(|c|
            (c.id().to_string(), Utc.timestamp(c.time().seconds(), 0))
        );

        let res = t.ok();
        res
    });
    return p;
}

/// Get all the local branches for the passed in repository
fn get_all_local_branches(repo: &Repository) -> git2::Branches {
    let branches_list = match repo.branches(Option::Some(BranchType::Local)) {
        Ok(br) => br,
        Err(e) => panic!("failed to init: {}", e),
    };

    return branches_list;
}