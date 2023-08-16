//! This program will make random commits to a GitHub repository in random hour intervals.

extern crate git2;

use chrono::{Timelike, Utc};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use git2::{Cred, PushOptions, RemoteCallbacks, Repository, Signature};
use rand::Rng;
use std::{fs::File, io::Write, path::Path, thread, time};

fn main() {
    let repo_path = Path::new("PATH/TO/YOUR/PRIVATE/REPO"); //TODO

    let mut rng = rand::thread_rng();
    let mut commit_hour: u32 = rng.gen_range(0..24);
    println!(
        "The program has started. The first commit will happen at hour: {}",
        commit_hour
    );

    loop {
        let current_hour = Utc::now().hour();
        if current_hour >= commit_hour - 1 && current_hour <= commit_hour + 1 {
            println!("It's time to commit.");
            let repo = match Repository::open(&repo_path) {
                Ok(repo) => repo,
                Err(e) => panic!("failed to open: {}", e),
            };

            let n: u32 = rng.gen_range(1..49);
            println!("Making {} commits.", n);
            for _i in 0..n {
                let file_path = repo_path.join("file.txt");
                let mut file = File::create(&file_path).unwrap();
                writeln!(file, "Random number: {}", rng.gen::<u32>()).unwrap();

                let mut index = repo.index().unwrap();
                index.add_path(Path::new("file.txt")).unwrap();
                index.write().unwrap();

                let tree_id = index.write_tree().unwrap();
                let tree = repo.find_tree(tree_id).unwrap();

                let head = repo.head().unwrap().peel_to_commit().unwrap();
                let signature = Signature::now("YOUR NAME", "YOUR EMAIL ADDRESS").unwrap(); //TODO

                let mut hasher = Sha256::new();

                hasher.input_str(&_i.to_string());

                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    &hasher.result_str(),
                    &tree,
                    &[&head],
                )
                .unwrap();

                let mut remote = repo.find_remote("origin").unwrap();

                let mut callbacks = RemoteCallbacks::new();
                callbacks.credentials(|_url, _username_from_url, _allowed_types| {
                    Cred::userpass_plaintext("YOUR USER NAME", 
                    "YOUR GITHUB ACCESS TOKEN, GET IT HERE: https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens") //TODO
                });

                let mut push_options = PushOptions::new();
                push_options.remote_callbacks(callbacks);

                let push_result = remote.push(
                    &["refs/heads/main:refs/heads/main"],
                    Some(&mut push_options),
                );

                match push_result {
                    Err(_e) => {
                        println!("Push failed, please check your internet connection.");
                        commit_hour += 1;
                        println!("I will try to push again at hour: {}", commit_hour);
                        break;
                    }
                    _ => {}
                }

                println!("Commit and push successful. #{}", _i);
            }

            if commit_hour != current_hour + 1 {
                commit_hour = rng.gen_range(commit_hour..24);
                println!("Next commit will happen at hour: {}", commit_hour);
            } else {
                println!("Next commit will happen in an hour!!");
            }
        }

        thread::sleep(time::Duration::from_secs(3600));
    }
}
