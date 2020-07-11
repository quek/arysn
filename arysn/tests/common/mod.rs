use arysn::prelude::*;
use arysn_macro::define_ar;

pub fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

define_ar!(User {
    table_name: users,
    has_many: roles
});

define_ar!(Role {
    table_name: roles,
    belongs_to: user
});
