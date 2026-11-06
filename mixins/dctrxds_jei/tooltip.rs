// The game is building a stack's tooltip: note which stack, to learn whether
// this can tell the slot under the mouse.
fn dctrxds_jei_tooltip(stack: &horizon_client::items::ItemStack) {
    crate::game::dctrxds_jei::tooltip_shown(&stack.item);
}
