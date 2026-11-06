// That folder's files.
fn dctrxds_jei_listener_files(
    listener: &crate::game::native_reload_listeners::NativeReloadListener,
    files: &[(String, serde_json::Value)],
) {
    crate::game::dctrxds_jei::listener_files(listener, files);
}
