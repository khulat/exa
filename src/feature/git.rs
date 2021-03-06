use std::old_path::GenericPath;
use std::old_path::posix::Path;

use ansi_term::{ANSIString, ANSIStrings};
use ansi_term::Colour::*;
use git2;

use file::GREY;

/// Container of Git statuses for all the files in this folder's Git repository.
pub struct Git {
    statuses: Vec<(Path, git2::Status)>,
}

impl Git {

    /// Discover a Git repository on or above this directory, scanning it for
    /// the files' statuses if one is found.
    pub fn scan(path: &Path) -> Result<Git, git2::Error> {
        use std::os::unix::ffi::OsStrExt;
        use std::ffi::AsOsStr;

        // TODO: libgit2-rs uses the new Path module, but exa still uses the
        // old_path one, and will have to continue to do so until the new IO
        // module gets a bit more developed. So we have to turn Paths into
        // old_path::Paths. Yes, this is hacky, but hopefully temporary.
        let new_path = path.as_os_str();
        let repo = try!(git2::Repository::discover(new_path));
        let workdir = match repo.workdir() {
            Some(w) => Path::new(w.as_os_str().as_bytes()),
            None => return Ok(Git { statuses: vec![] }),  // bare repo
        };

        let statuses = try!(repo.statuses(None)).iter()
                                                .map(|e| (workdir.join(e.path_bytes()), e.status()))
                                                .collect();
        Ok(Git { statuses: statuses })
    }

    /// Get the status for the file at the given path, if present.
    pub fn status(&self, path: &Path) -> String {
        let status = self.statuses.iter()
                                  .find(|p| &p.0 == path);
        match status {
            Some(&(_, s)) => ANSIStrings( &[Git::index_status(s), Git::working_tree_status(s) ]).to_string(),
            None => GREY.paint("--").to_string(),
        }
    }

    /// Get the combined status for all the files whose paths begin with the
    /// path that gets passed in. This is used for getting the status of
    /// directories, which don't really have an 'official' status.
    pub fn dir_status(&self, dir: &Path) -> String {
        let s = self.statuses.iter()
                             .filter(|p| dir.is_ancestor_of(&p.0))
                             .fold(git2::Status::empty(), |a, b| a | b.1);

        ANSIStrings( &[Git::index_status(s), Git::working_tree_status(s)] ).to_string()
    }

    /// The character to display if the file has been modified, but not staged.
    fn working_tree_status(status: git2::Status) -> ANSIString<'static> {
        match status {
            s if s.contains(git2::STATUS_WT_NEW) => Green.paint("A"),
            s if s.contains(git2::STATUS_WT_MODIFIED) => Blue.paint("M"),
            s if s.contains(git2::STATUS_WT_DELETED) => Red.paint("D"),
            s if s.contains(git2::STATUS_WT_RENAMED) => Yellow.paint("R"),
            s if s.contains(git2::STATUS_WT_TYPECHANGE) => Purple.paint("T"),
            _ => GREY.paint("-"),
        }
    }

    /// The character to display if the file has been modified, and the change
    /// has been staged.
    fn index_status(status: git2::Status) -> ANSIString<'static> {
        match status {
            s if s.contains(git2::STATUS_INDEX_NEW) => Green.paint("A"),
            s if s.contains(git2::STATUS_INDEX_MODIFIED) => Blue.paint("M"),
            s if s.contains(git2::STATUS_INDEX_DELETED) => Red.paint("D"),
            s if s.contains(git2::STATUS_INDEX_RENAMED) => Yellow.paint("R"),
            s if s.contains(git2::STATUS_INDEX_TYPECHANGE) => Purple.paint("T"),
            _ => GREY.paint("-"),
        }
    }
}
