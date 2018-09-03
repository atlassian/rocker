use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use shiplift::Docker;
use termion::event::{Event, Key};
use tui::{backend::MouseBackend, layout::Rect, widgets::Widget, Terminal};
use tui_logger::{Dispatcher, EventListener, TuiLoggerSmartWidget, TuiWidgetState};

use app::AppCommand;
use views::View;

pub struct AppLogsView {
    state: RefCell<TuiWidgetState>,
    dispatcher: Rc<RefCell<Dispatcher<Event>>>,
}

impl AppLogsView {
    pub fn new() -> AppLogsView {
        AppLogsView {
            state: RefCell::new(TuiWidgetState::new()),
            dispatcher: Rc::new(RefCell::new(Dispatcher::new())),
        }
    }
}

impl View for AppLogsView {
    fn handle_input(&mut self, key: Key, _docker: Arc<Docker>) -> Option<AppCommand> {
        if self.dispatcher.borrow_mut().dispatch(&Event::Key(key)) {
            Some(AppCommand::NoOp)
        } else {
            None
        }
    }

    fn refresh(&mut self, _docker: Arc<Docker>) {}

    fn draw(&self, t: &mut Terminal<MouseBackend>, rect: Rect) {
        TuiLoggerSmartWidget::default()
            .state(&*self.state.borrow_mut())
            .dispatcher(self.dispatcher.clone())
            .render(t, &rect);
    }
}
