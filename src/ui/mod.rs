use std::{cell::RefCell, ops::Deref, rc::Rc};

pub struct AppUi {
    data: Rc<AppData>,
    default_handler: RefCell<Option<nwg::EventHandler>>,
}

#[derive(Default)]
pub struct AppData {
    engine: crate::PlaybackEngine,
    window: nwg::Window,
    layout: nwg::GridLayout,
    start_record_btn: nwg::Button,
    stop_record_btn: nwg::Button,
    start_playback_btn: nwg::Button,
    stop_playback_btn: nwg::Button,
}

impl nwg::NativeUi<AppUi> for AppData {
    fn build_ui(mut data: AppData) -> Result<AppUi, nwg::NwgError> {
        nwg::Window::builder()
            .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
            .size((300, 135))
            .position((300, 300))
            .title("Midi playback app")
            .build(&mut data.window)?;

        nwg::Button::builder()
            .text("Start recording")
            .parent(&data.window)
            .build(&mut data.start_record_btn)?;

        nwg::Button::builder()
            .text("Stop recording")
            .parent(&data.window)
            .build(&mut data.stop_record_btn)?;

        nwg::Button::builder()
            .text("Start playback")
            .parent(&data.window)
            .build(&mut data.start_playback_btn)?;

        nwg::Button::builder()
            .text("Stop playback")
            .parent(&data.window)
            .build(&mut data.stop_playback_btn)?;

        let ui = AppUi {
            data: Rc::new(data),
            default_handler: Default::default(),
        };

        let evt_ui = Rc::downgrade(&ui.data);
        let handle_events = move |evt, _evt_data, handle| {
            if let Some(ui) = evt_ui.upgrade() {
                match evt {
                    nwg::Event::OnButtonClick => {
                        if &handle == &ui.start_record_btn {
                            if let Err(err) = ui.engine.start_record() {
                                tracing::error!("error starting recording: {:?}", err);
                            }
                        } else if &handle == &ui.stop_record_btn {
                            ui.engine.stop_record();
                        } else if &handle == &ui.start_playback_btn {
                            if let Err(err) = ui.engine.start_playback() {
                                tracing::error!("error starting playback: {:?}", err);
                            }
                        } else if &handle == &ui.stop_playback_btn {
                            ui.engine.stop_playback();
                        }
                    }
                    nwg::Event::OnWindowClose => {
                        if &handle == &ui.window {
                            nwg::stop_thread_dispatch();
                        }
                    }
                    _ => {}
                }
            }
        };

        *ui.default_handler.borrow_mut() = Some(nwg::full_bind_event_handler(
            &ui.window.handle,
            handle_events,
        ));

        nwg::GridLayout::builder()
            .parent(&ui.window)
            .spacing(1)
            .child(0, 0, &ui.start_record_btn)
            .child(1, 0, &ui.stop_record_btn)
            .child(0, 1, &ui.start_playback_btn)
            .child(1, 1, &ui.stop_playback_btn)
            .build(&ui.layout)?;

        return Ok(ui);
    }
}

impl Drop for AppUi {
    fn drop(&mut self) {
        let handler = self.default_handler.borrow();
        if handler.is_some() {
            nwg::unbind_event_handler(handler.as_ref().unwrap());
        }
    }
}

impl Deref for AppUi {
    type Target = AppData;

    fn deref(&self) -> &AppData {
        &self.data
    }
}
