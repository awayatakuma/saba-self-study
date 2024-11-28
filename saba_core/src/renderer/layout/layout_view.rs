use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use crate::constants::CONTENT_AREA_WIDTH;
use crate::display_item::DisplayItem;
use crate::renderer::dom::api::get_target_element_node;
use crate::renderer::dom::node::Node;
use crate::renderer::layout::layout_object::LayoutObject;
use crate::renderer::{css::cssom::StyleSheet, dom::node::ElementKind};

use super::layout_object::{create_layout_object, LayoutObjectKind, LayoutPoint, LayoutSize};

#[derive(Debug, Clone)]
pub struct LayoutView {
    root: Option<Rc<RefCell<LayoutObject>>>,
}

impl LayoutView {
    pub fn new(root: Rc<RefCell<Node>>, cssom: &StyleSheet) -> Self {
        let body_root = get_target_element_node(Some(root), ElementKind::Body);

        let mut tree = Self {
            root: build_layout_tree(&body_root, &None, cssom),
        };

        tree.update_layout();

        tree
    }

    pub fn root(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.root.clone()
    }

    fn update_layout(&mut self) {
        Self::calculate_node_size(&self.root, LayoutSize::new(CONTENT_AREA_WIDTH, 0));

        Self::calculate_node_position(
            &self.root,
            LayoutPoint::new(0, 0),
            LayoutObjectKind::Block,
            None,
            None,
        )
    }

    fn calculate_node_position(
        node: &Option<Rc<RefCell<LayoutObject>>>,
        parent_point: LayoutPoint,
        previous_sibling_kind: LayoutObjectKind,
        previous_sibling_point: Option<LayoutPoint>,
        previous_sibling_size: Option<LayoutSize>,
    ) {
        if let Some(n) = node {
            n.borrow_mut().compute_position(
                parent_point,
                previous_sibling_kind,
                previous_sibling_point,
                previous_sibling_size,
            );

            let first_child = n.borrow().first_child();
            Self::calculate_node_position(
                &first_child,
                n.borrow().point(),
                LayoutObjectKind::Block,
                None,
                None,
            );

            let next_sibling = n.borrow().next_sibling();
            Self::calculate_node_position(
                &next_sibling,
                parent_point,
                n.borrow().kind(),
                Some(n.borrow().point()),
                Some(n.borrow().size()),
            );
        }
    }

    fn calculate_node_size(node: &Option<Rc<RefCell<LayoutObject>>>, parent_size: LayoutSize) {
        if let Some(n) = node {
            if n.borrow().kind() == LayoutObjectKind::Block {
                n.borrow_mut().compute_size(parent_size);
            }

            let first_child = n.borrow().first_child();
            Self::calculate_node_size(&first_child, n.borrow().size());

            let next_sibling = n.borrow().next_sibling();
            Self::calculate_node_size(&next_sibling, parent_size);

            n.borrow_mut().compute_size(parent_size);
        }
    }

    fn paint_node(node: &Option<Rc<RefCell<LayoutObject>>>, display_items: &mut Vec<DisplayItem>) {
        match node {
            Some(n) => {
                display_items.extend(n.borrow_mut().paint());

                let first_child = n.borrow().first_child();
                Self::paint_node(&first_child, display_items);

                let next_sibling = n.borrow().next_sibling();
                Self::paint_node(&next_sibling, display_items);
            }
            None => (),
        }
    }

    pub fn paint(&self) -> Vec<DisplayItem> {
        let mut display_items = Vec::new();

        Self::paint_node(&self.root, &mut display_items);

        display_items
    }

    pub fn find_node_by_position(&self, position: (i64, i64)) -> Option<Rc<RefCell<LayoutObject>>> {
        Self::find_node_by_position_internal(&self.root(), position)
    }

    fn find_node_by_position_internal(
        node: &Option<Rc<RefCell<LayoutObject>>>,
        position: (i64, i64),
    ) -> Option<Rc<RefCell<LayoutObject>>> {
        match node {
            Some(n) => {
                let first_child = n.borrow().first_child();
                let result1 = Self::find_node_by_position_internal(&first_child, position);

                if result1.is_some() {
                    return result1;
                }

                let next_sibling = n.borrow().next_sibling();
                let result2 = Self::find_node_by_position_internal(&next_sibling, position);

                if result2.is_some() {
                    return result2;
                }

                if n.borrow().point().x() <= position.0
                    && position.0 <= (n.borrow().point().x() + n.borrow().size().width())
                    && n.borrow().point().y() <= position.1
                    && position.1 <= (n.borrow().point().y() + n.borrow().size().height())
                {
                    return Some(n.clone());
                }
                None
            }
            None => None,
        }
    }
}

fn build_layout_tree(
    node: &Option<Rc<RefCell<Node>>>,
    parent_obj: &Option<Rc<RefCell<LayoutObject>>>,
    cssom: &StyleSheet,
) -> Option<Rc<RefCell<LayoutObject>>> {
    let mut target_node = node.clone();
    let mut layout_object = create_layout_object(node, parent_obj, cssom);
    while layout_object.is_none() {
        if let Some(n) = target_node {
            target_node = n.borrow().next_sibling().clone();
            layout_object = create_layout_object(&target_node, parent_obj, cssom);
        } else {
            return layout_object;
        }
    }

    if let Some(n) = target_node {
        let original_first_child = n.borrow().first_child();
        let original_next_sibling = n.borrow().next_sibling();
        let mut first_child = build_layout_tree(&original_first_child, &layout_object, cssom);
        let mut next_sibling = build_layout_tree(&original_next_sibling, &None, cssom);

        if first_child.is_none() && original_first_child.is_some() {
            let mut original_dom_node = original_first_child
                .expect("first child should exist")
                .borrow()
                .next_sibling();
            loop {
                first_child = build_layout_tree(&original_dom_node, &layout_object, cssom);

                if first_child.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node
                        .expect("next sibling should exist")
                        .borrow()
                        .next_sibling();
                    continue;
                }

                break;
            }
        }
        if next_sibling.is_none() && n.borrow().next_sibling().is_some() {
            let mut original_dom_node = original_next_sibling
                .expect("first child should exist")
                .borrow()
                .next_sibling();

            loop {
                next_sibling = build_layout_tree(&original_dom_node, &None, cssom);

                if next_sibling.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node
                        .expect("next sibling should exist")
                        .borrow()
                        .next_sibling();
                    continue;
                }

                break;
            }
        }

        let obj = match layout_object {
            Some(ref obj) => obj,
            None => panic!("render object should exist here"),
        };

        obj.borrow_mut().set_first_child(first_child);
        obj.borrow_mut().set_next_sibling(next_sibling);
    }

    layout_object
}
