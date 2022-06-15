use std::path::{Path, PathBuf};

/// Efficient way to join paths with fewer allocations than PathBuf.join().join().join()...
pub fn path_join(components: &[&Path]) -> PathBuf {
    let total_size_required: usize = components.iter().map(|c| 1 + c.as_os_str().len()).sum::<usize>().saturating_sub(1);
    let mut result = PathBuf::with_capacity(total_size_required);
    for c in components {
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> Vec<&'static Path> {
        vec![
            Path::new(""),
            Path::new("/"),
            Path::new("/absolute"),
            Path::new("relative"),
            Path::new("/absolute/path"),
            Path::new("relative/path"),
            Path::new("/absolute/path/with//double//slash"),
            Path::new("relative/path/with//double//slash"),
        ]
    }

    #[test]
    fn empty_components() {
        assert_eq!(Path::new(""), path_join(&[]).as_path());
    }

    #[test]
    fn one_component() {
        for first in paths() {
            assert_eq!(first, path_join(&[first]).as_path());
        }
    }

    #[test]
    fn two_components() {
        for first in paths() {
            for second in paths() {
                assert_eq!(first.join(second), path_join(&[first, second]));
            }
        }
    }

    #[test]
    fn three_components() {
        for first in paths() {
            for second in paths() {
                for third in paths() {
                    assert_eq!(first.join(second).join(third), path_join(&[first, second, third]));
                }
            }
        }
    }

    #[test]
    fn four_components() {
        for first in paths() {
            for second in paths() {
                for third in paths() {
                    for fourth in paths() {
                        assert_eq!(first.join(second).join(third).join(fourth), path_join(&[first, second, third, fourth]));
                    }
                }
            }
        }
    }
}
