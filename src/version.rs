use anyhow::Result;
use git_conventional::Commit;
use semver::Version;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BumpType {
    None,
    Patch,
    Minor,
    Major,
}

pub fn calculate_next_version(current_version: &Version, commits: &[String]) -> Result<(Version, BumpType)> {
    let mut bump = BumpType::None;

    for message in commits {
        // Parse the commit message
        // git-conventional expects the full commit message
        if let Ok(commit) = Commit::parse(message.trim()) {
            if commit.breaking() {
                bump = BumpType::Major;
                break; // Major is the highest, we can stop checking if we just want the next version
            }

            match commit.type_().as_str() {
                "feat" => {
                    if bump < BumpType::Minor {
                        bump = BumpType::Minor;
                    }
                }
                "fix" => {
                    if bump < BumpType::Patch {
                        bump = BumpType::Patch;
                    }
                }
                "chore" => {
                    bump = BumpType::Patch;
                    break;
                }
                _ => {
                    // Other types (docs, style, refactor, etc.) usually imply patch or no bump
                    // semantic-release usually bumps patch for these if configured, but default is often just fix/feat
                    // Let's stick to fix=patch, feat=minor for now.
                    // If we want to be safe, any valid conventional commit that isn't feat/breaking could be patch?
                    // Standard semantic-release:
                    // fix -> patch
                    // feat -> minor
                    // breaking -> major
                    // others -> no release (unless configured)
                }
            }
        }
    }

    let mut next_version = current_version.clone();
    match bump {
        BumpType::Major => {
            next_version.major += 1;
            next_version.minor = 0;
            next_version.patch = 0;
        }
        BumpType::Minor => {
            next_version.minor += 1;
            next_version.patch = 0;
        }
        BumpType::Patch => {
            next_version.patch += 1;
        }
        BumpType::None => {}
    }

    Ok((next_version, bump))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_next_version_major() {
        let current_version = Version::parse("1.2.3").unwrap();
        let commits = vec!["chore!: rewrite everything".to_string()];
        let (next_version, bump) = calculate_next_version(&current_version, &commits).unwrap();
        
        assert_eq!(bump, BumpType::Major);
        assert_eq!(next_version, Version::parse("2.0.0").unwrap());
    }

    #[test]
    fn test_calculate_next_version_fix() {
        let current_version = Version::parse("1.2.3").unwrap();
        let commits = vec!["fix: bug fix".to_string()];
        let (next_version, bump) = calculate_next_version(&current_version, &commits).unwrap();
        
        assert_eq!(bump, BumpType::Patch);
        assert_eq!(next_version, Version::parse("1.2.4").unwrap());
    }

    #[test]
    fn test_calculate_next_version_feat() {
        let current_version = Version::parse("1.2.3").unwrap();
        let commits = vec!["feat: new feature".to_string()];
        let (next_version, bump) = calculate_next_version(&current_version, &commits).unwrap();
        
        assert_eq!(bump, BumpType::Minor);
        assert_eq!(next_version, Version::parse("1.3.0").unwrap());
    }
}
