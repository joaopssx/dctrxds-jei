// A key goes down over the open screen.
fn dctrxds_jei_key(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    key: &str,
    modifiers: u32,
    canceled: &mut bool,
) {
    if game.native_on(crate::game::dctrxds_jei::ID) {
        crate::game::dctrxds_jei::key(game, screen, key, modifiers, canceled);
    }
}
