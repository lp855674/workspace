use crate::ShellTheme;
use gpui::{
    AnyElement, App, CursorStyle, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px, rgb,
};
use gpui_component::{Icon, IconName, Size, Sizable, tooltip::Tooltip};
use std::rc::Rc;

#[derive(Clone, Copy)]
pub enum WorkbenchButtonVariant {
    Subtle,
    Selected,
}

#[derive(Clone, Copy)]
pub enum WorkbenchIconScale {
    XSmall,
    Small,
    Medium,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusButtonIcon {
    Sidebar,
    Terminal,
    RightDock,
}

pub fn button_variant_colors(
    variant: WorkbenchButtonVariant,
    theme: ShellTheme,
) -> (u32, u32, u32) {
    match variant {
        WorkbenchButtonVariant::Subtle => (
            theme.status_bar_button,
            theme.border,
            theme.title_bar_foreground,
        ),
        WorkbenchButtonVariant::Selected => (
            theme.status_bar_button_active,
            theme.status_bar_button_hover,
            theme.title_bar_foreground,
        ),
    }
}

pub fn workbench_icon(icon: IconName, scale: WorkbenchIconScale, color: u32) -> AnyElement {
    let icon = match scale {
        WorkbenchIconScale::XSmall => Icon::new(icon).with_size(Size::XSmall),
        WorkbenchIconScale::Small => Icon::new(icon).with_size(Size::Small),
        WorkbenchIconScale::Medium => Icon::new(icon).with_size(Size::Medium),
    };
    icon.text_color(rgb(color)).into_any_element()
}

pub fn workbench_asset_icon(
    path: &'static str,
    scale: WorkbenchIconScale,
    color: u32,
) -> AnyElement {
    let icon = match scale {
        WorkbenchIconScale::XSmall => Icon::new(Icon::default().path(path)).with_size(Size::XSmall),
        WorkbenchIconScale::Small => Icon::new(Icon::default().path(path)).with_size(Size::Small),
        WorkbenchIconScale::Medium => Icon::new(Icon::default().path(path)).with_size(Size::Medium),
    };
    icon.text_color(rgb(color)).into_any_element()
}

pub fn status_button_icon_path(icon: StatusButtonIcon, _active: bool) -> &'static str {
    match icon {
        StatusButtonIcon::Sidebar => "icons/tool_folder.svg",
        StatusButtonIcon::Terminal => "icons/tool_terminal.svg",
        StatusButtonIcon::RightDock => "icons/zed_assistant.svg",
    }
}

pub fn toolbar_labeled_button(
    id: &'static str,
    icon: Option<IconName>,
    label: &'static str,
    tooltip: &'static str,
    active: bool,
    on_click: Rc<dyn Fn(&mut Window, &mut App)>,
    theme: ShellTheme,
    emphasized: bool,
) -> impl IntoElement {
    let (background, _border, foreground) = button_variant_colors(
        if active {
            WorkbenchButtonVariant::Selected
        } else {
            WorkbenchButtonVariant::Subtle
        },
        theme,
    );
    div()
        .id(id)
        .h(px(28.))
        .min_w(px(56.))
        .px_2()
        .flex()
        .items_center()
        .justify_center()
        .gap_1()
        .rounded(px(0.))
        .border_0()
        .bg(rgb(background))
        .text_xs()
        .text_color(if active || emphasized {
            rgb(foreground)
        } else {
            rgb(theme.status_bar_active)
        })
        .hover(move |style| {
            style
                .bg(rgb(theme.status_bar_button_hover))
                .text_color(rgb(theme.title_bar_foreground))
        })
        .cursor(CursorStyle::PointingHand)
        .tooltip(move |window, cx| Tooltip::new(tooltip).build(window, cx))
        .on_click(move |_, window, cx| {
            on_click(window, cx);
        })
        .when_some(icon, |this, icon| {
            this.child(workbench_icon(icon, WorkbenchIconScale::Small, foreground))
        })
        .child(label)
}

pub fn toolbar_icon_button(
    id: &'static str,
    icon: StatusButtonIcon,
    tooltip: &'static str,
    active: bool,
    on_click: Rc<dyn Fn(&mut Window, &mut App)>,
    theme: ShellTheme,
) -> impl IntoElement {
    let (background, _border, foreground) = button_variant_colors(
        if active {
            WorkbenchButtonVariant::Selected
        } else {
            WorkbenchButtonVariant::Subtle
        },
        theme,
    );
    div()
        .id(id)
        .h(px(28.))
        .w(px(30.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(0.))
        .border_0()
        .bg(rgb(background))
        .text_color(rgb(if active {
            foreground
        } else {
            theme.status_bar_active
        }))
        .hover(move |style| {
            style
                .bg(rgb(theme.status_bar_button_hover))
                .text_color(rgb(theme.title_bar_foreground))
        })
        .cursor(CursorStyle::PointingHand)
        .tooltip(move |window, cx| Tooltip::new(tooltip).build(window, cx))
        .on_click(move |_, window, cx| {
            on_click(window, cx);
        })
        .child(workbench_asset_icon(
            status_button_icon_path(icon, active),
            WorkbenchIconScale::Medium,
            foreground,
        ))
}

pub fn header_icon_button(
    id: &'static str,
    icon: IconName,
    tooltip: &'static str,
    on_click: Rc<dyn Fn(&mut Window, &mut App)>,
    theme: ShellTheme,
) -> impl IntoElement {
    let (background, border, foreground) =
        button_variant_colors(WorkbenchButtonVariant::Selected, theme);
    div()
        .id(id)
        .h(px(24.))
        .w(px(24.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .border_1()
        .border_color(rgb(border))
        .bg(rgb(background))
        .text_color(rgb(foreground))
        .hover(move |style| {
            style
                .bg(rgb(theme.status_bar_button_hover))
                .border_color(rgb(theme.status_bar_active))
                .text_color(rgb(theme.title_bar_foreground))
        })
        .cursor(CursorStyle::PointingHand)
        .tooltip(move |window, cx| Tooltip::new(tooltip).build(window, cx))
        .on_click(move |_, window, cx| {
            on_click(window, cx);
        })
        .child(workbench_icon(icon, WorkbenchIconScale::Medium, foreground))
}

pub fn toolbar_static_icon(
    id: &'static str,
    icon: IconName,
    tooltip: impl Into<String>,
    theme: ShellTheme,
) -> impl IntoElement {
    let tooltip = tooltip.into();
    div()
        .id(id)
        .h(px(24.))
        .w(px(28.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(4.))
        .text_color(rgb(theme.status_bar_foreground))
        .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
        .child(workbench_icon(
            icon,
            WorkbenchIconScale::Small,
            theme.status_bar_foreground,
        ))
}

pub fn terminal_tab_close_button(
    index: usize,
    active: bool,
    on_click: Rc<dyn Fn(&mut Window, &mut App)>,
    theme: ShellTheme,
) -> impl IntoElement {
    let (background, border, foreground) = button_variant_colors(
        if active {
            WorkbenchButtonVariant::Selected
        } else {
            WorkbenchButtonVariant::Subtle
        },
        theme,
    );

    div()
        .id(("terminal-tab-close", index))
        .h(px(16.))
        .w(px(16.))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(3.))
        .border_1()
        .border_color(rgb(border))
        .bg(rgb(background))
        .hover(move |style| {
            style
                .bg(rgb(theme.status_bar_button_hover))
                .border_color(rgb(theme.status_bar_active))
        })
        .on_click(move |_, window, cx| {
            on_click(window, cx);
        })
        .child(workbench_icon(
            IconName::Close,
            WorkbenchIconScale::Small,
            foreground,
        ))
}

pub fn corner_toolbar_group(children: Vec<AnyElement>, theme: ShellTheme) -> impl IntoElement {
    let (background, border, _) = button_variant_colors(WorkbenchButtonVariant::Subtle, theme);
    children
        .into_iter()
        .enumerate()
        .fold(
            div()
                .h(px(28.))
                .flex()
                .items_center()
                .rounded(px(6.))
                .border_1()
                .border_color(rgb(border))
                .bg(rgb(background)),
            |group, (index, child)| {
                group.child(
                    div()
                        .flex()
                        .items_center()
                        .when(index > 0, |this| {
                            this.child(
                                div()
                                    .h(px(16.))
                                    .w(px(1.))
                                    .bg(rgb(border)),
                            )
                        })
                        .child(child),
                )
            },
        )
}

pub fn titlebar_icon_button(
    id: &'static str,
    icon: IconName,
    active: bool,
    theme: ShellTheme,
) -> impl IntoElement {
    let (background, _border, foreground) = button_variant_colors(
        if active {
            WorkbenchButtonVariant::Selected
        } else {
            WorkbenchButtonVariant::Subtle
        },
        theme,
    );
    div()
        .id(id)
        .h(px(24.))
        .min_w(px(24.))
        .px_1()
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(5.))
        .bg(rgb(background))
        .text_color(rgb(foreground))
        .hover(move |style| style.bg(rgb(theme.window_control_hover)))
        .child(workbench_icon(icon, WorkbenchIconScale::Medium, foreground))
}

pub fn titlebar_tab_chip(
    label: &str,
    active: bool,
    theme: ShellTheme,
) -> impl IntoElement {
    let (background, border, foreground) = button_variant_colors(
        if active {
            WorkbenchButtonVariant::Selected
        } else {
            WorkbenchButtonVariant::Subtle
        },
        theme,
    );
    div()
        .h(px(22.))
        .max_w(px(168.))
        .px_1()
        .mb(px(-1.))
        .flex()
        .items_center()
        .rounded_t(px(4.))
        .border_1()
        .border_color(rgb(border))
        .bg(rgb(background))
        .text_xs()
        .text_color(rgb(if active {
            foreground
        } else {
            theme.sidebar_muted
        }))
        .hover(move |style| style.bg(rgb(theme.window_control_hover)))
        .child(
            div()
                .max_w(px(142.))
                .overflow_hidden()
                .text_color(rgb(if active {
                    foreground
                } else {
                    theme.sidebar_muted
                }))
                .truncate()
                .child(label.to_owned()),
        )
}

pub fn titlebar_label(label: &str, theme: ShellTheme) -> impl IntoElement {
    div()
        .h(px(24.))
        .px_1()
        .flex()
        .items_center()
        .text_xs()
        .text_color(rgb(theme.title_bar_foreground))
        .font_weight(FontWeight::SEMIBOLD)
        .child(label.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{status_button_icon_path, StatusButtonIcon};

    #[test]
    fn status_toolbar_buttons_use_zed_asset_icons() {
        assert_eq!(
            status_button_icon_path(StatusButtonIcon::Sidebar, true),
            "icons/tool_folder.svg"
        );
        assert_eq!(
            status_button_icon_path(StatusButtonIcon::Terminal, false),
            "icons/tool_terminal.svg"
        );
        assert_eq!(
            status_button_icon_path(StatusButtonIcon::RightDock, false),
            "icons/zed_assistant.svg"
        );
        assert_eq!(
            status_button_icon_path(StatusButtonIcon::RightDock, true),
            "icons/zed_assistant.svg"
        );
    }
}
