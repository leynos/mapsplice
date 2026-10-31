//! Property test for layout-independent JSON export.

use proptest::prelude::*;

use crate::{
    export_workspace::export_source,
    model::{COMPACT, SPACED, WRAPPED, render, roadmap_model},
};

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        .. ProptestConfig::default()
    })]

    #[test]
    fn layout_variants_export_identical_json(model in roadmap_model()) {
        let mut exports = Vec::new();
        for layout in [COMPACT, WRAPPED, SPACED] {
            let source = render(&model, layout);
            let exported = export_source(&source)
                .unwrap_or_else(|error| panic!("layout {layout:?} failed to export: {error}"));
            exports.push((layout, exported));
        }
        let Some((first_layout, first_export)) = exports.first() else {
            prop_assert!(false, "at least one layout should be rendered");
            return Ok(());
        };
        for (layout, exported) in exports.iter().skip(1) {
            prop_assert_eq!(
                exported,
                first_export,
                "layout {:?} differs from layout {:?}",
                layout,
                first_layout
            );
        }
    }
}
