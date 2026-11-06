// A screen closes.
fn dctrxds_jei_closing(screen: &horizon_client::mod_api::NativeScreenView<'_>) {
    crate::game::dctrxds_jei::screen_closed(screen);
}
