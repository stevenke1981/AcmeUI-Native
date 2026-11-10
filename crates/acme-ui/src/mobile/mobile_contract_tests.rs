//! Default and structural compatibility contracts, not interaction certification.
use crate::WidgetNode;
use acme_core::NodeId;
use acme_layout::{LayoutKind, WidgetLayoutContext};
fn shape(name: &str, node: WidgetNode<()>, kind: LayoutKind, children: usize) {
    let context = WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    };
    let layout = node.to_layout_with_context(NodeId::new(1), &context);
    assert_eq!(layout.style.kind, kind, "{name}: layout kind");
    assert_eq!(node.children().len(), children, "{name}: widget children");
    assert_eq!(layout.children.len(), children, "{name}: layout children");
}
// One registry emits both the named tests and the expected public inventory.
// Adding a module requires adding its actual constructor/default contract here.
macro_rules! contracts {
    ($($name:ident: ($builder:expr,$kind:ident,$count:expr,|$b:ident|$defaults:block)),+ $(,)?) => {
        $(#[test] fn $name() {
            let $b=$builder;
            $defaults
            shape(stringify!($name),$b.into(),LayoutKind::$kind,$count);
        })+
        #[test] fn every_public_mobile_module_has_a_contract() {
            let mut expected=vec![$(stringify!($name)),+]; expected.sort_unstable();
            let mut actual:Vec<_>=include_str!("mod.rs").lines()
                .filter_map(|line|line.trim().strip_prefix("pub mod ")?.strip_suffix(';')).collect();
            actual.sort_unstable();
            assert_eq!(actual,expected,"public modules must have named default/shape contracts");
        }
    };
}
contracts! {
    mobile_action: (super::mobile_action::<()>("Action"),Row,1,|b| {
        assert_eq!(b.id.as_str(),"mobile_action"); assert_eq!(b.label,"Action");
        assert!(b.icon.is_none() && !b.destructive && !b.disabled && b.on_press.is_none());
    }),
    mobile_avatar: (super::mobile_avatar::<()>("AB"),Stack,1,|b| {
        assert_eq!(b.id.as_str(),"mobile_avatar");assert_eq!(b.initials,"AB");
        assert_eq!(b.size,super::MobileAvatarSize::Md);
    }),
    mobile_banner: (super::mobile_banner::<()>("Banner"),Row,2,|b| {
        assert_eq!(b.id.as_str(),"mobile_banner");assert_eq!(b.text,"Banner");
        assert_eq!(b.tone,super::MobileBannerTone::Info);
        assert!(b.action_label.is_none() && b.on_action.is_none() && b.on_dismiss.is_none());
    }),
    mobile_button: (super::mobile_button::<()>("mobile_button","Tap"),Leaf,0,|b| {
        assert_eq!(b.id.as_str(),"mobile_button");assert_eq!(b.text,"Tap");
        assert_eq!(b.size,super::MobileButtonSize::Md);assert!(!b.disabled && b.on_press.is_none());
    }),
    mobile_card: (super::mobile_card::<()>("Card"),Column,1,|b| {
        assert_eq!(b.id.as_str(),"mobile_card");assert_eq!(b.title,"Card");
        assert!(b.subtitle.is_none() && b.media.is_none() && b.elevated);
    }),
    mobile_chip: (super::mobile_chip::<()>("Chip"),Row,1,|b| {
        assert_eq!(b.id.as_str(),"mobile_chip");assert_eq!(b.text,"Chip");
        assert!(!b.selected && b.on_dismiss.is_none() && b.on_tap.is_none());
    }),
    mobile_list_item: (super::mobile_list_item::<()>("Item",None::<&str>),Row,1,|b| {
        assert_eq!(b.id.as_str(),"mobile_list_item");assert_eq!(b.title,"Item");
        assert!(b.detail.is_none() && b.trailing_icon.is_none() && b.on_tap.is_none());
    }),
    mobile_loader: (super::mobile_loader::<()>(),Column,1,|b| {
        assert_eq!(b.id.as_str(),"mobile_loader");assert!(b.message.is_none());
    }),
    mobile_notification: (super::mobile_notification::<()>("Notice"),Column,1,|b| {
        assert_eq!(b.id.as_str(),"mobile_notification");assert_eq!(b.title,"Notice");
        assert!(b.body.is_none() && b.action_label.is_none() && b.on_action.is_none() && b.on_dismiss.is_none());
    }),
    mobile_progress: (super::mobile_progress::<()>(2.0,10.0),Column,2,|b| {
        assert_eq!(b.id.as_str(),"mobile_progress");assert_eq!((b.value,b.total),(2.0,10.0));assert!(b.label.is_none());
    }),
    mobile_search: (super::mobile_search::<()>("Find"),Row,2,|b| {
        assert_eq!(b.id.as_str(),"mobile_search");assert_eq!(b.placeholder,"Find");
        assert!(b.value.is_empty() && b.on_input.is_none() && b.on_submit.is_none() && b.on_clear.is_none());
    }),
    mobile_segmented: (super::mobile_segmented::<()>(vec!["A","B"]),Row,2,|b| {
        assert_eq!(b.id.as_str(),"mobile_segmented");assert_eq!(b.segments,vec!["A","B"]);
        assert_eq!(b.selected_index,0);assert!(!b.disabled && b.on_select.is_none());
    }),
    mobile_sheet_handle: (super::mobile_sheet_handle::<()>(),Row,0,|b| {
        assert_eq!(b.id.as_str(),"mobile_sheet_handle");assert_eq!(b.width,36.0);
    }),
    mobile_stepper: (super::mobile_stepper::<()>(5),Row,3,|b| {
        assert_eq!(b.id.as_str(),"mobile_stepper");assert_eq!((b.value,b.min,b.max,b.step),(5,0,100,1));
        assert!(!b.disabled && b.on_decrement.is_none() && b.on_increment.is_none() && b.on_change.is_none());
    }),
    mobile_toggle: (super::mobile_toggle::<()>("Wi-Fi"),Stack,2,|b| {
        assert_eq!(b.id.as_str(),"mobile_toggle");assert_eq!(b.label,"Wi-Fi");
        assert!(!b.checked && !b.disabled && b.on_change.is_none() && b.on_toggle.is_none());
    }),
    action_sheet: (super::action_sheet::<()>("action_sheet"),Column,1,|b| {
        assert_eq!(b.id.as_str(),"action_sheet");assert!(b.title.is_none() && b.actions.is_empty() && b.show_cancel && b.on_select.is_none());
        assert_eq!(b.cancel_label,"Cancel");let item=super::ActionItem::new("Action");
        assert_eq!(item.label,"Action");assert!(!item.destructive && !item.disabled);
    }),
    bottom_nav: (super::bottom_nav::<()>("bottom_nav"),Row,0,|b| {
        assert_eq!(b.id.as_str(),"bottom_nav");assert!(b.items.is_empty());assert_eq!(b.selected_index,0);
        let item=super::bottom_nav_item::<()>("item","Home");
        assert_eq!(item.id.as_str(),"item");assert_eq!(item.label,"Home");
        assert!(item.icon.is_none() && !item.selected && item.on_select.is_none());
    }),
    bottom_sheet: (super::bottom_sheet::<()>("bottom_sheet"),Column,1,|b| {
        assert_eq!(b.id.as_str(),"bottom_sheet");assert!(b.title.is_none() && !b.open && b.child.is_none() && b.on_close.is_none());
    }),
    pull_to_refresh: (super::pull_to_refresh::<()>("pull_to_refresh"),Column,1,|b| {
        assert_eq!(b.id.as_str(),"pull_to_refresh");assert_eq!(b.progress,0.0);
        assert!(!b.refreshing && b.child.is_none() && b.on_refresh.is_none());
    }),
    search_bar: (super::search_bar::<()>("search_bar"),Row,2,|b| {
        assert_eq!(b.id.as_str(),"search_bar");assert_eq!(b.placeholder,"Search…");
        assert!(b.value.is_empty() && b.show_cancel && !b.auto_focus && b.on_submit.is_none() && b.on_change.is_none());
    }),
}
