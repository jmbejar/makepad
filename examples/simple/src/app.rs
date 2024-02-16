use makepad_widgets::*;
 
live_design!{
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*; 
    App = {{App}} {

        ui: <Window>{
            show_bg: true
            width: Fill,
            height: Fill
            
            draw_bg: {
                fn pixel(self) -> vec4 {
                    //return #000
                    return mix(#7, #3, self.pos.y);
                }
            }
            
            body = <View>{
                 
                flow: Down,
                spacing: 20,
                align: {
                    x: 0.5,
                    y: 0.5
                },
                button1 = <Button> {
                    text: "Hello world"
                }
                input1 = <TextInput> {
                    width: 100, height: 30
                    text: "Click to count"
                }
                
                label1 = <Label> {
                    draw_text: {
                        color: #f
                        text_style: {
                            font_size: (11),
                            font: {path: dep("crate://self/resources/Inter-Regular.ttf")}
                        }
                    }
                    text: "Qwen (abbr. for Tongyi Qianwen 通义千问) refers to the large language model family built by Alibaba Cloud"
                }

                <Label> {
                    width: 400
                    draw_text: {
                        color: #f
                        text_style: {
                            font_size: (11),
                            font: {path: dep("crate://self/resources/Inter-Regular.ttf")}
                        }
                    }
                    text: "Qwen (abbr. for Tongyi Qianwen 通义千问) refers to the large language model family built by Alibaba Cloud"
                }
                <Html>{
                    text:"this<a href='thing'>is basichtml</a><tag prop/>text<bold/>"
                }
            }
        }
    }
}
 
app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
    #[rust] counter: usize,
 }

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);
    }
}

impl MatchEvent for App{
    fn handle_actions(&mut self, cx: &mut Cx, actions:&Actions){
        if self.ui.button(id!(button1)).clicked(&actions) {
            log!("BUTTON CLICKED {}", self.counter); 
            self.counter += 1;
            let label = self.ui.label(id!(label1));
            label.set_text_and_redraw(cx,&format!("Counter: {}", self.counter));
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}