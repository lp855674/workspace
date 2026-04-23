use gpui::{AnyElement, FontWeight, IntoElement, div, prelude::*, px, rgb};
use gpui_component::{
    Icon, IconName, Sizable,
    list::ListItem,
    tree::{TreeEntry, tree},
};
use status_bar::StatusBarItem;

pub struct ShellSidebar {
    pub workspace_name: String,
    pub project_root: String,
    pub tree: AnyElement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellDockHost {
    pub id: &'static str,
    pub title: &'static str,
    pub active_panel: Option<String>,
    pub visible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellWorkspace {
    pub center: ShellDockHost,
    pub right: ShellDockHost,
    pub bottom: ShellDockHost,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShellStatus {
    pub left_text: String,
    pub right_text: String,
    pub items: Vec<StatusBarItem>,
}

pub fn render_shell(
    sidebar: ShellSidebar,
    workspace: ShellWorkspace,
    status: ShellStatus,
) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(rgb(0x0f1419))
        .text_color(gpui::white())
        .child(
            div()
                .flex()
                .flex_1()
                .min_h_0()
                .child(render_sidebar(sidebar))
                .child(render_workspace(workspace)),
        )
        .child(render_status_bar(status))
}

fn render_sidebar(sidebar: ShellSidebar) -> impl IntoElement {
    div()
        .w(px(255.))
        .h_full()
        .flex()
        .flex_col()
        .bg(rgb(0x121921))
        .border_r_1()
        .border_color(rgb(0x1c2730))
        .child(
            div()
                .w_full()
                .px_3()
                .pt_3()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(sidebar.workspace_name),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x8092a2))
                        .child(sidebar.project_root),
                ),
        )
        .child(div().flex_1().min_h_0().px_2().py_2().child(sidebar.tree))
        .child(
            div()
                .w_full()
                .px_3()
                .pb_3()
                .flex()
                .justify_between()
                .items_center()
                .text_xs()
                .text_color(rgb(0x8092a2))
                .child("Explorer"),
        )
}

fn render_workspace(workspace: ShellWorkspace) -> impl IntoElement {
    div().flex_1().min_w_0().min_h_0().flex().flex_col().child(
        div()
            .flex()
            .flex_1()
            .min_h_0()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .child(render_dock_host(workspace.center, DockHostKind::Center))
                    .when(workspace.bottom.visible, |this| {
                        this.child(render_dock_host(workspace.bottom, DockHostKind::Bottom))
                    }),
            )
            .when(workspace.right.visible, |this| {
                this.child(render_dock_host(workspace.right, DockHostKind::Right))
            }),
    )
}

#[derive(Clone, Copy)]
enum DockHostKind {
    Center,
    Right,
    Bottom,
}

fn render_dock_host(host: ShellDockHost, kind: DockHostKind) -> impl IntoElement {
    let active_panel = host.active_panel.unwrap_or_else(|| "empty".to_owned());
    let mut container = div()
        .bg(rgb(0x0f1419))
        .border_color(rgb(0x1c2730))
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(34.))
                .px_3()
                .flex()
                .items_center()
                .justify_between()
                .border_b_1()
                .border_color(rgb(0x1c2730))
                .bg(rgb(0x10161d))
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(host.title),
                )
                .child(div().text_xs().text_color(rgb(0x8092a2)).child(host.id)),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .p_3()
                .text_sm()
                .text_color(rgb(0xcdd8e2))
                .child(active_panel),
        );

    container = match kind {
        DockHostKind::Center => container.flex_1().min_w_0().min_h_0(),
        DockHostKind::Right => container.w(px(320.)).h_full().border_l_1(),
        DockHostKind::Bottom => container.h(px(200.)).border_t_1(),
    };

    container
}

fn render_status_bar(status: ShellStatus) -> impl IntoElement {
    div()
        .h(px(24.))
        .w_full()
        .px_3()
        .flex()
        .items_center()
        .justify_between()
        .bg(rgb(0x0c1116))
        .border_t_1()
        .border_color(rgb(0x1b242d))
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x9ab0c2))
                        .child(status.left_text),
                )
                .children(status.items.iter().take(2).map(|item| {
                    div()
                        .text_xs()
                        .text_color(rgb(0x8092a2))
                        .child(item.text.clone())
                })),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .children(
                    status
                        .items
                        .into_iter()
                        .skip(2)
                        .map(|item| div().text_xs().text_color(rgb(0x8092a2)).child(item.text)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x9ab0c2))
                        .child(status.right_text),
                ),
        )
}

pub fn render_file_tree(
    tree_state: &gpui::Entity<gpui_component::tree::TreeState>,
) -> impl IntoElement {
    tree(tree_state, |index, entry: &TreeEntry, selected, _, _| {
        let item = entry.item();
        let icon = if entry.is_folder() {
            if entry.is_expanded() {
                Icon::new(IconName::FolderOpen)
            } else {
                Icon::new(IconName::FolderClosed)
            }
        } else {
            Icon::new(IconName::File)
        };

        ListItem::new(index).selected(selected).px_2().py_1().child(
            div()
                .pl(px(12.0) * entry.depth() as f32)
                .flex()
                .items_center()
                .gap_2()
                .child(icon.small().text_color(rgb(0x8aa0b4)))
                .child(
                    div()
                        .text_sm()
                        .text_color(if item.is_disabled() {
                            rgb(0x556775)
                        } else {
                            rgb(0xcdd8e2)
                        })
                        .child(item.label.clone()),
                ),
        )
    })
}
