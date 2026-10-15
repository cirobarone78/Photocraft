//! The Layers panel's blend mode, Opacity, Fill and lock controls change every selected layer,
//! as in Photoshop, not only the active one.

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use serde_json::json;

use crate::PhotocraftApp;

fn app() -> Harness<'static, PhotocraftApp> {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).with_max_steps(64).build_eframe(|cc| {
        PhotocraftApp::setup_context(&cc.egui_ctx, Default::default());
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"width": 64, "height": 64, "background": "white"})).unwrap();
        app.run("layer.new.layer", json!({})).unwrap();
        app
    });
    h.run_steps(6);
    h
}

/// The Layers panel's lock buttons change every selected layer (as do blend mode, Opacity and
/// Fill), not only the active one.
#[test]
fn the_layers_panel_locks_every_selected_layer() {
    let mut h = app();
    let ids: Vec<u64> = h.state().session.active().unwrap().doc.layers.iter().map(|l| l.id.0).collect();
    let extra = h.state_mut().run("layer.new.layer", json!({})).unwrap()["layer"].as_u64().unwrap();
    h.state_mut().run("layer.select", json!({"layer": ids[1], "mode": "add"})).unwrap();
    h.run_steps(3);
    assert_eq!(h.state().session.active().unwrap().selected_layers().len(), 2);
    h.query_by_label_contains("Lock all").expect("the Lock all button").click();
    h.run_steps(3);
    let st = h.state().session.active().unwrap();
    let locked = |id: u64| st.doc.layer(photocraft_doc::LayerId(id)).unwrap().locks.all;
    assert!(locked(extra) && locked(ids[1]), "both selected layers lock");
    assert!(!locked(ids[0]), "the unselected Background doesn't");
}

/// The Basic layout's single lock toggle also locks every selected layer.
#[test]
fn the_basic_lock_toggle_locks_every_selected_layer() {
    let mut h = Harness::builder().with_size(egui::vec2(1400.0, 900.0)).with_max_steps(64).build_eframe(|cc| {
        PhotocraftApp::setup_context(&cc.egui_ctx, crate::theme::ThemeKind::Studio);
        let mut app = PhotocraftApp::new(photocraft_engine::Session::new(), crate::Services::default());
        app.run("prefs.set", json!({"path": "interface.theme", "value": crate::theme::ThemeKind::Studio.id()})).unwrap();
        app.ui.theme = crate::theme::ThemeKind::Studio;
        app.run("file.new", json!({"width": 64, "height": 64, "background": "white"})).unwrap();
        app
    });
    let a = h.state_mut().run("layer.new.layer", json!({})).unwrap()["layer"].as_u64().unwrap();
    let b = h.state_mut().run("layer.new.layer", json!({})).unwrap()["layer"].as_u64().unwrap();
    h.state_mut().run("layer.select", json!({"layer": a, "mode": "add"})).unwrap();
    h.run_steps(4);
    h.get_by_label("Lock layer").click();
    h.run_steps(3);
    let st = h.state().session.active().unwrap();
    let locked = |id: u64| st.doc.layer(photocraft_doc::LayerId(id)).unwrap().locks.all;
    assert!(locked(a) && locked(b), "both selected layers lock");
}

/// With the Background among the selected layers, blend mode, Opacity and Fill are greyed (as
/// for the Background alone): changing them would turn it into Layer 0.
#[test]
fn the_background_in_a_selection_greys_opacity() {
    let mut h = app();
    let st = h.state().session.active().unwrap();
    let (bg, top) = (st.doc.layers[0].id.0, st.active_layer.unwrap().0);
    let greyed = |h: &Harness<'static, PhotocraftApp>| h.get_all_by_label("Opacity:").any(|n| n.accesskit_node().is_disabled());
    assert!(!greyed(&h), "a normal layer alone");
    // The Background first, then the layer added: the layer is active, both are selected.
    h.state_mut().run("layer.select", json!({"layer": bg})).unwrap();
    h.state_mut().run("layer.select", json!({"layer": top, "mode": "add"})).unwrap();
    h.run_steps(3);
    assert_eq!(h.state().session.active().unwrap().selected_layers().len(), 2);
    assert_eq!(h.state().session.active().unwrap().active_layer.map(|l| l.0), Some(top));
    assert!(greyed(&h), "the Background is selected too");
}
