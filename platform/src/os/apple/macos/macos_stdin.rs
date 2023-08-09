use {
    std::{
        sync::{Arc, Mutex},
        cell::{RefCell},
        io,
        io::prelude::*,
        io::{BufReader},
    },
    crate::{
        makepad_live_id::*,
        makepad_math::*,
        makepad_error_log::*,
        makepad_micro_serde::*,
        event::Event,
        window::CxWindowPool,
        event::WindowGeom,
        texture::Texture,
        live_traits::LiveNew,
        thread::Signal,
        os::{
            metal_xpc::{
                xpc_service_proxy,
                xpc_service_proxy_poll_run_loop,
                fetch_xpc_service_texture,
            },
            metal::{MetalCx, DrawPassMode},
            cx_stdin::{HostToStdin, StdinToHost},
        },
        pass::{CxPassParent, PassClearColor, CxPassColorTexture},
        cx_api::{CxOsOp},
        cx::{Cx},
    }
};

impl Cx {
    
    pub (crate) fn stdin_handle_repaint(&mut self, metal_cx: &mut MetalCx) {
        let mut passes_todo = Vec::new();
        self.compute_pass_repaint_order(&mut passes_todo);
        self.repaint_id += 1;
        for pass_id in &passes_todo {
            match self.passes[*pass_id].parent.clone() {
                CxPassParent::Window(_) => {
                    self.draw_pass(*pass_id, metal_cx, DrawPassMode::StdinMain);
                    let _ = io::stdout().write_all(StdinToHost::DrawComplete.to_json().as_bytes());
                }
                CxPassParent::Pass(_) => {
                    //let dpi_factor = self.get_delegated_dpi_factor(parent_pass_id);
                    self.draw_pass(*pass_id, metal_cx, DrawPassMode::Texture);
                },
                CxPassParent::None => {
                    self.draw_pass(*pass_id, metal_cx, DrawPassMode::Texture);
                }
            }
        }
    }
    
    pub fn stdin_event_loop(&mut self, metal_cx: &mut MetalCx) {
        let _ = io::stdout().write_all(StdinToHost::ReadyToStart.to_json().as_bytes());
        let fb_shared = Arc::new(Mutex::new(RefCell::new(None)));
        let mut shared_check = 0;
        let fb_texture = Texture::new(self);
        let service_proxy = xpc_service_proxy();
        let mut reader = BufReader::new(std::io::stdin());
        let mut window_size = None;
        
        self.call_event_handler(&Event::Construct);
        
        loop {
            let mut line = String::new();
            if let Ok(len) = reader.read_line(&mut line) {
                if len == 0 {
                    break
                }
                // alright lets put the line in a json parser
                let parsed: Result<HostToStdin, DeJsonErr> = DeJson::deserialize_json(&line);
                
                match parsed {
                    Ok(msg) => match msg {
                        HostToStdin::ReloadFile{file:_, contents:_}=>{
                            // alright lets reload this file in our DSL system
                            
                        }
                        HostToStdin::MouseDown(e) => {
                            self.fingers.process_tap_count(
                                dvec2(e.x,e.y),
                                e.time
                            );
                            self.fingers.mouse_down(e.button);

                            self.call_event_handler(&Event::MouseDown(e.into()));
                        }
                        HostToStdin::MouseMove(e) => {
                            self.call_event_handler(&Event::MouseMove(e.into()));
                            self.fingers.cycle_hover_area(live_id!(mouse).into());
                            self.fingers.switch_captures();
                        }
                        HostToStdin::MouseUp(e) => {
                            let button = e.button;
                            self.call_event_handler(&Event::MouseUp(e.into()));
                            self.fingers.mouse_up(button);
                            self.fingers.cycle_hover_area(live_id!(mouse).into());
                        }
                        HostToStdin::Scroll(e) => {
                            self.call_event_handler(&Event::Scroll(e.into()))
                        }
                        HostToStdin::WindowSize(ws) => {
                            if window_size != Some(ws) {
                                window_size = Some(ws);
                                self.redraw_all();
                                
                                let window = &mut self.windows[CxWindowPool::id_zero()];
                                window.window_geom = WindowGeom {
                                    dpi_factor: ws.dpi_factor,
                                    inner_size: dvec2(ws.width, ws.height),
                                    ..Default::default()
                                };
                                self.stdin_handle_platform_ops(metal_cx, &fb_texture);
                            }
                        }
                        HostToStdin::Tick {frame: _, time} => if let Some(ws) = window_size {
                            // poll the service for updates
                            let uid = if let Some((_, uid)) = fb_shared.lock().unwrap().borrow().as_ref() {*uid}else {0};
                            fetch_xpc_service_texture(service_proxy.as_id(), 0, uid, Box::new({
                                let fb_shared = fb_shared.clone();
                                move | shared_handle,
                                shared_uid | {
                                    *fb_shared.lock().unwrap().borrow_mut() = Some((shared_handle, shared_uid));
                                }
                            }));
        
                            // check signals
                            if Signal::check_and_clear_ui_signal(){
                                self.handle_media_signals();
                                self.call_event_handler(&Event::Signal);
                            }
                            if self.check_live_file_watcher(){
                                self.call_event_handler(&Event::LiveEdit);
                                self.redraw_all();
                            }
                            self.handle_networking_events();
                            
                            // alright a tick.
                            // we should now run all the stuff.
                            if self.new_next_frames.len() != 0 {
                                self.call_next_frame_event(time);
                            }
                            
                            if self.need_redrawing() {
                                self.call_draw_event();
                                self.mtl_compile_shaders(metal_cx);
                            }
                            
                            // lets render to the framebuffer
                            if let Some((shared_handle, shared_uid)) = fb_shared.lock().unwrap().borrow().as_ref() {
                                if shared_check != *shared_uid {
                                    shared_check = *shared_uid;
                                    let window = &mut self.windows[CxWindowPool::id_zero()];
                                    let pass = &mut self.passes[window.main_pass_id.unwrap()];
                                    pass.paint_dirty = true;
                                    // alright so. lets update the shared texture
                                    let fb_texture = &mut self.textures[fb_texture.texture_id()];
                                    fb_texture.os.update_from_shared_handle(
                                        metal_cx,
                                        shared_handle.as_id(),
                                        (ws.width * ws.dpi_factor) as u64,
                                        (ws.height * ws.dpi_factor) as u64
                                    );
                                }
                            }
                            // we need to make this shared texture handle into a true metal one
                            self.stdin_handle_repaint(metal_cx);
                        }
                    }
                    Err(err) => { // we should output a log string
                        error!("Cant parse stdin-JSON {} {:?}", line, err);
                    }
                }
            }
            // we should poll our runloop
            self.stdin_handle_platform_ops(metal_cx, &fb_texture);
            xpc_service_proxy_poll_run_loop();
        }
    }
    
    
    fn stdin_handle_platform_ops(&mut self, _metal_cx: &MetalCx, main_texture: &Texture) {
        while let Some(op) = self.platform_ops.pop() {
            match op {
                CxOsOp::CreateWindow(window_id) => {
                    if window_id != CxWindowPool::id_zero() {
                        panic!("ONLY ONE WINDOW SUPPORTED");
                    }
                    let window = &mut self.windows[CxWindowPool::id_zero()];
                    window.is_created = true;
                    // lets set up our render pass target
                    let pass = &mut self.passes[window.main_pass_id.unwrap()];
                    pass.color_textures = vec![CxPassColorTexture {
                        clear_color: PassClearColor::ClearWith(vec4(1.0,1.0,0.0,1.0)),
                        //clear_color: PassClearColor::ClearWith(pass.clear_color),
                        texture_id: main_texture.texture_id()
                    }];
                },
                _ => ()
                /*
                CxOsOp::CloseWindow(_window_id) => {},
                CxOsOp::MinimizeWindow(_window_id) => {},
                CxOsOp::MaximizeWindow(_window_id) => {},
                CxOsOp::RestoreWindow(_window_id) => {},
                CxOsOp::FullscreenWindow(_window_id) => {},
                CxOsOp::NormalizeWindow(_window_id) => {}
                CxOsOp::SetTopmost(_window_id, _is_topmost) => {}
                CxOsOp::XrStartPresenting(_) => {},
                CxOsOp::XrStopPresenting(_) => {},
                CxOsOp::ShowTextIME(_area, _pos) => {},
                CxOsOp::HideTextIME => {},
                CxOsOp::SetCursor(_cursor) => {},
                CxOsOp::StartTimer {timer_id, interval, repeats} => {},
                CxOsOp::StopTimer(timer_id) => {},
                CxOsOp::StartDragging(dragged_item) => {}
                CxOsOp::UpdateMenu(menu) => {}*/
            }
        }
    }
    
}
