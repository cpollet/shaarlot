use vergen::{vergen, Config, ShaKind};

fn main() {
    let mut config = Config::default();
    *config.git_mut().sha_kind_mut() = ShaKind::Both;
    vergen(config).unwrap();

    build_data::set_GIT_BRANCH();
    build_data::set_GIT_COMMIT();
    build_data::set_GIT_DIRTY();
    build_data::set_SOURCE_TIMESTAMP();
    build_data::no_debug_rebuilds();
}
