// The wheel turns over the open screen.
fn dctrxds_jei_scroll(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    scroll: [f64; 2],
    canceled: &mut bool,
) {
    if game.native_on(crate::game::dctrxds_jei::ID) {
        crate::game::dctrxds_jei::scroll(screen, scroll, canceled);
    }
}
