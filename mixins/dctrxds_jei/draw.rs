// Over the open screen, after it and its tooltips.
fn dctrxds_jei_draw(
    game: &crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
) {
    if game.native_on(crate::game::dctrxds_jei::ID) {
        crate::game::dctrxds_jei::draw(game, batch, screen);
    }
}
