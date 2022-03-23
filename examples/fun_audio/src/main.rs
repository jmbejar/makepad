pub use makepad_component::{self, *};
pub use makepad_platform::{self, *, audio::*, midi::*};

mod piano;
mod audio;
use crate::piano::*;
use crate::audio::*;

live_register!{
    use AudioComponent::*;
    use FrameComponent::*;
    use makepad_component::theme::*;
    use makepad_component::frame::*;
    use makepad_platform::shader::std::*;
    
    const HEADER_RADIUS_CLOSED: vec2(3.0, 3.0)
    const HEADER_RADIUS_OPEN: vec2(3.0, 1.0)
    
    MainHeader: FoldHeader {
        closed_state: {apply: {
            header: {bg: {radius: (HEADER_RADIUS_CLOSED)}}
        }}
        opened_state: {apply: {
            header: {bg: {radius: (HEADER_RADIUS_OPEN)}}
        }}
        header: BoxY {
            mouse_cursor: Default,
            bg: {color: #6},
            width: Fill
            layout: {flow: Right, padding: 8, spacing: 5}
        }
    }
    
    SubHeader: FoldHeader {
        header: Rect {
            mouse_cursor: Default,
            bg: {color: #5},
            width: Fill
            height: Fit
            layout: {flow: Right, padding: 8, spacing: 5}
        }
    }
    
    FoldablePiano: MainHeader {
        header: {
            fold_button:= FoldButton {}
            label:= Label {text: "Keys"}
        }
        closed_state: {apply: {
            body: {g1: {bg: {color: #0000}}}
        }}
        opened_state: {apply: {
            body: {g1: {bg: {color: #000a}}}
        }}
        body: Frame {
            layout: {flow: Overlay}
            width: Fit
            height: Fit
            Frame {
                layout: {flow: Down}
                width: Fit
                height: Fit
                piano: = Piano {}
                GradientY {
                    width: Fill
                    height: 10
                    bg: {color: #000a, color2: #0000}
                }
            }
            g1:= GradientY {
                width: Fill
                height: 2
                bg: {color: #000a, color2: #0000, no_v_scroll: true}
            }
        }
    }
    
    App: {{App}} {
        window: {pass: {clear_color: (COLOR_BG_APP)}}
        audio_graph: {
            root: Mixer {
                /*c0: BasicSynth {
                    plugin: "AUMIDISynth"
                    preset_data: "21adslkfjalkwqwe"
                }*/
                c1: Instrument {
                    key_range: {start: 34, end: 47 shift: 30}
                    PluginEffect {
                        plugin: "AUReverb2"
                    }
                    PluginMusicDevice {
                        plugin: "AUMIDISynth"
                    }
                }
            }
        }
        
        frame: {
            bg: {color: (COLOR_BG_APP)},
            walk: {width: Fill, height: Fill}
            layout: {
                padding: 8
                align: {x: 0.0, y: 0.0}
                spacing: 5.,
                flow: Flow::Down
            },
            Frame {
                layout: {flow: Right, spacing: 5.0}
                walk: {margin: {left: 60}, height: Fit}
                Button {label: "+  Band"}
                Button {label: "<"}
                Button {label: ">"}
                Solid {
                    width: Fill
                    height: 36
                    bg: {
                        const WAVE_HEIGHT: 0.15
                        const WAVE_FREQ: 0.2
                        fn pixel(self) -> vec4 {
                            let offset_y = 1.5
                            let pos2 = vec2(self.pos.x, self.pos.y + WAVE_HEIGHT * sin(WAVE_FREQ * self.pos.x * self.rect_size.x))
                            let sdf = Sdf2d::viewport(pos2 * self.rect_size)
                            sdf.clear(#2f)
                            sdf.move_to(0., self.rect_size.y * 0.5)
                            sdf.line_to(self.rect_size.x, self.rect_size.y * 0.5)
                            return sdf.stroke(#f, 1.0)
                        }
                    }
                }
            }
            
            Splitter {
                align: SplitterAlign::FromEnd(300)
                walk: {width: Fill, height: Fill}
                a: FoldablePiano {}
                b: Box {
                    clip: true,
                    mouse_cursor: Default,
                    bg: {color: #4, radius: 3.0, border_width: 0.5, border_color: #3}
                    height: Fill
                    layout: {flow: Down, padding: 0.5}
                    MainHeader {
                        header: {
                            mouse_cursor: Hand,
                            label:= Label {text: "Instruments"}
                        }
                        body: SubHeader {
                            header: {
                                mouse_cursor: Default,
                                fold_button:= FoldButton {}
                                swatch:= Circle {
                                    width: 10,
                                    height: 10
                                    bg: {color: #f00}
                                }
                                label:= Label {text: "Instrument"}
                                Rect{bg:{color:#f00}, width:Fill, height:8}
                            }
                            body: Frame{
                                bg: {color: #f00},
                                width: Fill,
                                height: Fit,
                                layout: {flow: Down}
                                Rect {
                                    mouse_cursor: Default,
                                    bg: {color: #5},
                                    width: Fill
                                    height: Fit
                                    layout: {flow: Right, padding: 8, spacing: 5, align: {x: 0.0}}
                                    label:= Label {text: "Stack item"}
                                }
                            }
                        }
                    }
                }
            }
        }
        
        scroll_view: {
            h_show: true,
            v_show: true,
            view: {}
        }
    }
}
main_app!(App);

#[derive(Live, LiveHook)]
pub struct App {
    frame: Frame,
    audio_graph: AudioGraph,
    window: BareWindow,
    scroll_view: ScrollView,
}

impl App {
    pub fn live_register(cx: &mut Cx) {
        makepad_component::live_register(cx);
        crate::audio::live_register(cx);
        crate::piano::live_register(cx);
    }
    
    pub fn new_app(cx: &mut Cx) -> Self {
        Self::new_as_main_module(cx, &module_path!(), id!(App)).unwrap()
    }
    
    pub fn handle_event(&mut self, cx: &mut Cx, event: &mut Event) {
        
        //self.desktop_window.handle_event(cx, event);
        self.scroll_view.handle_event(cx, event);
        
        for item in self.frame.handle_event(cx, event) {
            match item.id {
                id!(piano) => if let PianoAction::Note {is_on, note_number, velocity} = item.action.cast() {
                    self.audio_graph.send_midi_1_data(Midi1Note {
                        is_on,
                        note_number,
                        channel: 0,
                        velocity
                    }.into());
                }
                _ => ()
            }
        }
        
        for action in self.audio_graph.handle_event(cx, event) {
            match action {
                AudioGraphAction::Midi1Data(data) => if let Midi1Event::Note(note) = data.decode() {
                    let piano = self.frame.child_mut::<Piano>(id!(piano)).unwrap();
                    piano.set_note(cx, note.is_on, note.note_number)
                }
            }
        };

        match event {
            Event::KeyDown(ke) => {
                if let KeyCode::F1 = ke.key_code {
                }
                if let KeyCode::Escape = ke.key_code {
                }
                
            }
            Event::Draw(draw_event) => {
                self.draw(&mut Cx2d::new(cx, draw_event));
                //self.piano.set_key_focus(cx);
            }
            _ => ()
        }
    }
    
    pub fn draw(&mut self, cx: &mut Cx2d) {
        if self.window.begin(cx).is_err() {
            return;
        }
        
        while self.frame.draw(cx).is_err() {};
        
        self.window.end(cx);
    }
}