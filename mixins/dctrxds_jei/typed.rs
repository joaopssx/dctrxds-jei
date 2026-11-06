// A character is typed over the open screen.
fn dctrxds_jei_typed(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    c: char,
    canceled: &mut bool,
) {
    if game.native_on(crate::game::dctrxds_jei::ID) {
        crate::game::dctrxds_jei::typed(screen, c, canceled);
    }
}
