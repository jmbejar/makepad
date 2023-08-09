use {
    crate::{
        button::ButtonAction,
        makepad_draw::*,
        widget::*
    }
};

live_design!{
    import makepad_draw::shader::std::*;
    
    DrawDesktopButton= {{DrawDesktopButton}} {
        fn pixel(self) -> vec4 {
            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
            sdf.aa *= 3.0;
            let sz = 4.5;
            let c = self.rect_size * vec2(0.5, 0.5);
            
            // WindowsMin
            match self.button_type {
                DesktopButtonType::WindowsMin => {
                    sdf.clear(mix(#3, mix(#6, #9, self.pressed), self.hover));
                    sdf.move_to(c.x - sz, c.y);
                    sdf.line_to(c.x + sz, c.y);
                    sdf.stroke(#f, 0.5 + 0.5 * self.dpi_dilate);
                    return sdf.result;
                }
                DesktopButtonType::WindowsMax => {
                    sdf.clear(mix(#3, mix(#6, #9, self.pressed), self.hover));
                    sdf.rect(c.x - sz, c.y - sz, 2. * sz, 2. * sz);
                    sdf.stroke(#f, 0.5 + 0.5 * self.dpi_dilate);
                    return sdf.result;
                }
                DesktopButtonType::WindowsMaxToggled => {
                    let clear = mix(#3, mix(#6, #9, self.pressed), self.hover);
                    sdf.clear(clear);
                    let sz = 3.5;
                    sdf.rect(c.x - sz + 1., c.y - sz - 1., 2. * sz, 2. * sz);
                    sdf.stroke(#f, 0.5 + 0.5 * self.dpi_dilate);
                    sdf.rect(c.x - sz - 1., c.y - sz + 1., 2. * sz, 2. * sz);
                    sdf.fill_keep(clear);
                    sdf.stroke(#f, 0.5 + 0.5 * self.dpi_dilate);
                    return sdf.result;
                }
                DesktopButtonType::WindowsClose => {
                    sdf.clear(mix(#3, mix(#e00, #c00, self.pressed), self.hover));
                    sdf.move_to(c.x - sz, c.y - sz);
                    sdf.line_to(c.x + sz, c.y + sz);
                    sdf.move_to(c.x - sz, c.y + sz);
                    sdf.line_to(c.x + sz, c.y - sz);
                    sdf.stroke(#f, 0.5 + 0.5 * self.dpi_dilate);
                    return sdf.result;
                }
                DesktopButtonType::XRMode => {
                    sdf.clear(mix(#3, mix(#0aa, #077, self.pressed), self.hover));
                    let w = 12.;
                    let h = 8.;
                    sdf.box(c.x - w, c.y - h, 2. * w, 2. * h, 2.);
                    // subtract 2 eyes
                    sdf.circle(c.x - 5.5, c.y, 3.5);
                    sdf.subtract();
                    sdf.circle(c.x + 5.5, c.y, 3.5);
                    sdf.subtract();
                    sdf.circle(c.x, c.y + h - 0.75, 2.5);
                    sdf.subtract();
                    sdf.fill(#8);
                    
                    return sdf.result;
                }
                DesktopButtonType::Fullscreen => {
                    sz = 8.;
                    sdf.clear(mix(#3, mix(#6, #9, self.pressed), self.hover));
                    sdf.rect(c.x - sz, c.y - sz, 2. * sz, 2. * sz);
                    sdf.rect(c.x - sz + 1.5, c.y - sz + 1.5, 2. * (sz - 1.5), 2. * (sz - 1.5));
                    sdf.subtract();
                    sdf.rect(c.x - sz + 4., c.y - sz - 2., 2. * (sz - 4.), 2. * (sz + 2.));
                    sdf.subtract();
                    sdf.rect(c.x - sz - 2., c.y - sz + 4., 2. * (sz + 2.), 2. * (sz - 4.));
                    sdf.subtract();
                    sdf.fill(#f); //, 0.5 + 0.5 * dpi_dilate);
                    
                    return sdf.result;
                }
            }
            return #f00;
        }
    }
    
    DesktopButton= {{DesktopButton}} {
        
        state:{
            hover = {
                default: off,
                off = {
                    from: {all: Forward {duration: 0.1}}
                    apply: {
                        draw_bg: {pressed: 0.0, hover: 0.0}
                    }
                }
                
                on = {
                    from: {
                        all: Forward {duration: 0.1}
                        state_down: Snap
                    }
                    apply: {
                        draw_bg: {
                            pressed: 0.0,
                            hover: 1.0,
                        }
                    }
                }
                
                pressed = {
                    from: {all: Snap}
                    apply: {
                        draw_bg: {
                            pressed: 1.0,
                            hover: 1.0,
                        }
                    }
                }
            }
        }
    }
}

#[derive(Live)]
pub struct DesktopButton {
    #[state] state: LiveState,
    #[live] walk: Walk,
    #[live] draw_bg: DrawDesktopButton,
}

impl Widget for DesktopButton{
   fn handle_widget_event_with(
        &mut self,
        cx: &mut Cx,
        event: &Event,
        dispatch_action: &mut dyn FnMut(&mut Cx, WidgetActionItem)
    ) {
        let uid = self.widget_uid();
        self.handle_event_with(cx, event, &mut | cx, action | {
            dispatch_action(cx, WidgetActionItem::new(action.into(),uid));
        });
    }

    fn get_walk(&self)->Walk{self.walk}
    
    fn redraw(&mut self, cx:&mut Cx){
        self.draw_bg.redraw(cx)
    }
    
    fn draw_walk_widget(&mut self, cx: &mut Cx2d, walk: Walk) -> WidgetDraw {
        let _ = self.draw_walk(cx, walk);
        WidgetDraw::done()
    }
}

#[derive(Live, LiveHook)]
#[live_ignore]
#[repr(u32)]
pub enum DesktopButtonType {
    WindowsMin = shader_enum(1),
    WindowsMax = shader_enum(2),
    WindowsMaxToggled = shader_enum(3),
    WindowsClose = shader_enum(4),
    XRMode = shader_enum(5),
    #[pick] Fullscreen = shader_enum(6),
}

#[derive(Live, LiveHook)]
#[repr(C)]
pub struct DrawDesktopButton {
    #[deref] draw_super: DrawQuad,
    #[live] hover: f32,
    #[live] pressed: f32,
    #[live] button_type: DesktopButtonType
}

impl LiveHook for DesktopButton {
    fn before_live_design(cx:&mut Cx){
        register_widget!(cx, DesktopButton)
    }
    
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        let (w, h) = match self.draw_bg.button_type {
            DesktopButtonType::WindowsMin
                | DesktopButtonType::WindowsMax
                | DesktopButtonType::WindowsMaxToggled
                | DesktopButtonType::WindowsClose => (46., 29.),
            DesktopButtonType::XRMode => (50., 36.),
            DesktopButtonType::Fullscreen => (50., 36.),
        };
        self.walk = Walk::fixed_size(dvec2(w, h))
    }
}

impl DesktopButton {
    pub fn handle_event_with(&mut self, cx: &mut Cx, event: &Event, dispatch_action: &mut dyn FnMut(&mut Cx, ButtonAction),) {
        self.state_handle_event(cx, event);

        match event.hits(cx, self.draw_bg.area()) {
            Hit::FingerDown(_fe) => {
                dispatch_action(cx, ButtonAction::Pressed);
                self.animate_state(cx, id!(hover.pressed));
            },
            Hit::FingerHoverIn(_) => {
                cx.set_cursor(MouseCursor::Hand);
                 self.animate_state(cx, id!(hover.on));
            }
            Hit::FingerHoverOut(_) => {
                self.animate_state(cx, id!(hover.off));
            }
            Hit::FingerUp(fe) => if fe.is_over {
                dispatch_action(cx, ButtonAction::Clicked);
                if fe.device.has_hovers() {
                    self.animate_state(cx, id!(hover.on));
                }
                else{
                    self.animate_state(cx, id!(hover.off));
                }
            }
            else {
                dispatch_action(cx, ButtonAction::Released);
                self.animate_state(cx, id!(hover.off));
            }
            _ => ()
        };
    }
    
    pub fn area(&mut self)->Area{
        self.draw_bg.area()
    }
    pub fn get_widget_walk(&self)->Walk{self.walk}
    
    pub fn draw_walk(&mut self, cx: &mut Cx2d, walk:Walk) {
        self.draw_bg.draw_walk(cx, walk);
    }
}
