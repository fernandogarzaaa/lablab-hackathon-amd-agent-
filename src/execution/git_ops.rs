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

    /// Create and checkout a new branch.
    pub fn create_branch(&self, name: &str) -> Result<()> {
        let mut branch = self.repo.find_branch(name, BranchType::Local)?;
        branch.set_upstream(Some(&format!("refs/remotes/origin/{}", name)))?;
        info!("Created branch: {}", name);
        Ok(())
    }

    /// Commit changes with a message.
    pub fn commit(&self, message: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"], git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let sig = self.repo.signature()?;
        let head_commit = self.repo.head()?.peel_to_commit()?;

        self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &[&head_commit],
        )?;
        info!("Committed: {}", message);
        Ok(())
    }

    /// Get the current HEAD commit hash.
    pub fn head(&self) -> Result<String> {
        Ok(self.repo.head()?.peel_to_commit()?.id().to_string())
    }
}
