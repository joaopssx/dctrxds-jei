// A mouse button goes down over the open screen.
fn dctrxds_jei_press(
    game: &crate::game::Game<'_>,
    screen: &horizon_client::mod_api::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    if game.native_on(crate::game::dctrxds_jei::ID) {
        crate::game::dctrxds_jei::press(screen, button, modifiers, canceled);
    }
}
