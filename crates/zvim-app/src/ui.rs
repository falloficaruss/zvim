use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    div, px, rgb, AppContext, Application, Context, IntoElement, ParentElement, Render, Styled,
    Window, WindowOptions,
};

use crate::app::{BootstrapError, ZvimApp};

pub fn run_app_shell() -> Result<(), BootstrapError> {
    let app = ZvimApp::bootstrap()?;
    let report = app.render_boot_report();
    let launch_error = Rc::new(RefCell::new(None));

    Application::new().run({
        let launch_error = Rc::clone(&launch_error);
        let report = report.clone();
        move |cx| {
            if let Err(error) = cx.open_window(WindowOptions::default(), |_window, cx| {
                cx.new(|_| RootView::new(report.clone()))
            }) {
                *launch_error.borrow_mut() = Some(BootstrapError::UiLaunch(error.to_string()));
                cx.quit();
            }
        }
    });

    if let Some(error) = launch_error.borrow_mut().take() {
        return Err(error);
    }

    Ok(())
}

pub struct RootView {
    report: String,
}

impl RootView {
    pub fn new(report: String) -> Self {
        Self { report }
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .bg(rgb(0x101418))
            .text_color(rgb(0xe6edf3))
            .font_family("Iosevka ZVIM")
            .p(px(24.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .child(
                        div()
                            .text_size(px(28.0))
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("ZVIM"),
                    )
                    .child(
                        div()
                            .text_size(px(16.0))
                            .text_color(rgb(0x8aa0b5))
                            .child("Clean, high-performance workspace bootstrap"),
                    )
                    .child(
                        div()
                            .mt(px(12.0))
                            .p(px(16.0))
                            .bg(rgb(0x151b23))
                            .border_1()
                            .border_color(rgb(0x28323d))
                            .rounded(px(12.0))
                            .child(self.report.clone()),
                    ),
            )
    }
}
