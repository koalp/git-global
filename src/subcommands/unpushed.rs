//! The `unpushed` subcommand: shows repositories that have commits not pushed to a remote
//!
// TODO:
// - (optionnal) show repos that don't have remote
// - (optionnal) only look for a subset of branches with `--branch`

use std::sync::{mpsc, Arc};
use std::thread;

use git2;

use config::Config;
use errors::Result;
use repo::Repo;
use report::Report;

/// Runs the `unpushed` subcommand.
pub fn execute(mut config: Config) -> Result<Report> {
    //let include_untracked = config.show_untracked;
    let repos = config.get_repos();
    let n_repos = repos.len();
    let mut report = Report::new(&repos);
    // TODO: limit number of threads, perhaps with mpsc::sync_channel(n)?
    let (tx, rx) = mpsc::channel();
    for repo in repos {
        let tx = tx.clone();
        let repo = Arc::new(repo);
        thread::spawn(move || {
            let path = repo.path();
            // TODO: replace this by a list of branch that aren't synced
            let synced = repo.is_origin_synced();
            tx.send((path, synced)).unwrap();
        });
    }
    for _ in 0..n_repos {
        let (path, synced) = rx.recv().unwrap();
        let repo = Repo::new(path.to_string());
        if !synced {
            report.add_repo_message(&repo, format!(""));
        }
    }
    Ok(report)
}
