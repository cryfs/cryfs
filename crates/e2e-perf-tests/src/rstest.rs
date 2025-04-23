use rstest_reuse::{self, *};

#[template]
pub fn all_atime_behaviors(
    #[values(
        AtimeUpdateBehavior::Noatime,
        AtimeUpdateBehavior::Strictatime,
        AtimeUpdateBehavior::Relatime,
        AtimeUpdateBehavior::NodiratimeRelatime,
        AtimeUpdateBehavior::NodiratimeStrictatime
    )]
    atime_behavior: AtimeUpdateBehavior,
) {
}
