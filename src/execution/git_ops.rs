//! GitOps — Git operations (clone, branch, commit).

use anyhow::Result;
use git2::{Repository, BranchType};
use tracing::info;

/// Git operations wrapper around libgit2.
pub struct GitOps {
    repo: Repository,
}

impl GitOps {
    /// Clone a repository to a local path.
    pub fn clone(url: &str, path: &str) -> Result<Self> {
        let repo = Repository::clone(url, path)?;
        info!("Cloned {} to {}", url, path);
        Ok(Self { repo })
    }

    /// Open an existing repository.
    pub fn open(path: &str) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    /// Create and checkout a new branch (or return Ok if it already exists).
    pub fn create_branch(&self, name: &str) -> Result<()> {
        // Check if branch already exists
        if self.repo.find_branch(name, BranchType::Local).is_ok() {
            info!("Branch already exists: {}", name);
            return Ok(());
        }

        // Get commit to branch from
        let commit = self.repo.head()?.peel_to_commit()?;

        // Create the branch
        let _branch = self.repo.branch(name, &commit, false)?;

        // Checkout the new branch - just update HEAD for now
        // (working directory state is handled separately by the caller)
        let _ = _branch; // silence unused variable warning

        // Update HEAD
        self.repo.set_head(&format!("refs/heads/{}", name))?;

        info!("Created and checked out branch: {}", name);
        Ok(())
    }

    /// Commit changes with a message.
    pub fn commit(&self, message: &str) -> Result<()> {
        let mut index = self.repo.index()?;

        // Recursively add all files
        index.add_all(["**/*"], git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let sig = self.repo.signature()?;

        // Handle initial commit (no HEAD yet)
        if let Ok(head) = self.repo.head() {
            let commit = head.peel_to_commit()?;
            self.repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                message,
                &tree,
                &[&commit],
            )?;
        } else {
            self.repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                message,
                &tree,
                &[],
            )?;
        }
        info!("Committed: {}", message);
        Ok(())
    }

    /// Get the current HEAD commit hash.
    pub fn head(&self) -> Result<String> {
        Ok(self.repo.head()?.peel_to_commit()?.id().to_string())
    }
}
