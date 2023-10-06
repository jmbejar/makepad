use {
    std::collections::hash_map::HashMap,
    crate::{
        makepad_derive_widget::*,
        makepad_draw::*,
        widget::*,
        scroll_bars::ScrollBars,
    },
};

live_design!{
    ViewBase = {{View}} {}
}

// maybe we should put an enum on the bools like

#[derive(Live, LiveHook)]
#[live_ignore]
pub enum ViewOptimize {
    #[pick] None,
    DrawList,
    Texture
}


#[derive(Live, LiveHook)]
#[live_ignore]
pub enum EventOrder {
    Down,
    #[pick] Up,
    #[live(Default::default())] List(Vec<LiveId>),
}


impl ViewOptimize {
    fn is_texture(&self) -> bool {
        if let Self::Texture = self {true} else {false}
    }
    fn is_draw_list(&self) -> bool {
        if let Self::DrawList = self {true} else {false}
    }
    fn needs_draw_list(&self) -> bool {
        return self.is_texture() || self.is_draw_list()
    }
}

#[derive(Live)]
pub struct View { // draw info per UI element
    #[live] draw_bg: DrawColor,
    
    #[live(false)] show_bg: bool,
    
    #[layout] layout: Layout,
    
    #[walk] walk: Walk,
    
    //#[live] use_cache: bool,
    #[live] dpi_factor: Option<f64>,
    
    #[live] optimize: ViewOptimize,
    #[live] event_order: EventOrder,
    
    #[live(true)] visible: bool,
    
    #[live(true)] grab_key_focus: bool,
    #[live(false)] block_signal_event: bool,
    #[live] cursor: Option<MouseCursor>,
    #[live] scroll_bars: Option<LivePtr>,
    #[live(false)] design_mode: bool,
    
    #[rust] find_cache: HashMap<u64, WidgetSet>,
    
    #[rust] scroll_bars_obj: Option<Box<ScrollBars >>,
    #[rust] view_size: Option<DVec2>,
    
    #[rust] area: Area,
    #[rust] draw_list: Option<DrawList2d>,
    
    #[rust] texture_cache: Option<ViewTextureCache>,
    #[rust] defer_walks: Vec<(LiveId, DeferWalk)>,
    #[rust] draw_state: DrawStateWrap<DrawState>,
    #[rust] children: ComponentMap<LiveId, WidgetRef>,
    #[rust] draw_order: Vec<LiveId>,
    
    #[animator] animator: Animator,
}

struct ViewTextureCache {
    pass: Pass,
    _depth_texture: Texture,
    color_texture: Texture,
}

impl LiveHook for View {
    fn before_live_design(cx: &mut Cx) {
        register_widget!(cx, View)
    }
    
    fn before_apply(&mut self, _cx: &mut Cx, from: ApplyFrom, _index: usize, _nodes: &[LiveNode]) {
        if let ApplyFrom::UpdateFromDoc {..} = from {
            //self.children.clear();
            self.draw_order.clear();
            self.find_cache.clear();
        }
    }
    
    fn after_apply(&mut self, cx: &mut Cx, _from: ApplyFrom, _index: usize, _nodes: &[LiveNode]) {
        if self.optimize.needs_draw_list() && self.draw_list.is_none() {
            self.draw_list = Some(DrawList2d::new(cx));
        }
        if self.scroll_bars.is_some() {
            if self.scroll_bars_obj.is_none() {
                self.scroll_bars_obj = Some(Box::new(ScrollBars::new_from_ptr(cx, self.scroll_bars)));
            }
        }
        /*
        if let Some(image_texture) = &mut self.image_texture {
            if self.image_scale != 0.0 {
                let texture_desc = image_texture.get_desc(cx);
                self.walk = Walk::fixed_size(
                    DVec2 {
                        x: texture_desc.width.unwrap() as f64 * self.image_scale,
                        y: texture_desc.height.unwrap() as f64 * self.image_scale
                    }
                );
            }
        */
    }
    
    fn apply_value_instance(&mut self, cx: &mut Cx, from: ApplyFrom, index: usize, nodes: &[LiveNode]) -> usize {
        //! TODO
        // NOTE FOR LIVE RELOAD
        // the id is always unique
        // Draw order is never cleared.
        
        let id = nodes[index].id;
        match from {
            ApplyFrom::Animate | ApplyFrom::ApplyOver => {
                if let Some(component) = self.children.get_mut(&nodes[index].id) {
                    component.apply(cx, from, index, nodes)
                }
                else {
                    nodes.skip_node(index)
                }
            }
            ApplyFrom::NewFromDoc {..} | ApplyFrom::UpdateFromDoc {..} => {
                if nodes[index].origin.has_prop_type(LivePropType::Instance) {
                    self.draw_order.push(id);
                    return self.children.get_or_insert(cx, id, | cx | {
                        WidgetRef::new(cx)
                    })
                        .apply(cx, from, index, nodes);
                }
                else {
                    cx.apply_error_no_matching_field(live_error_origin!(), index, nodes);
                    nodes.skip_node(index)
                }
            }
            _ => {
                nodes.skip_node(index)
            }
        }
    }
}

#[derive(Clone, PartialEq, WidgetRef)]
pub struct ViewRef(WidgetRef);


#[derive(Clone, WidgetSet)]
pub struct ViewSet(WidgetSet);

#[derive(Clone, WidgetAction)]
pub enum ViewAction {
    None,
    FingerDown(FingerDownEvent),
    FingerUp(FingerUpEvent),
    FingerMove(FingerMoveEvent),
    KeyDown(KeyEvent),
    KeyUp(KeyEvent),
}

impl ViewRef {
    pub fn finger_down(&self, actions: &WidgetActions) -> Option<FingerDownEvent> {
        if let Some(item) = actions.find_single_action(self.widget_uid()) {
            if let ViewAction::FingerDown(fd) = item.action() {
                return Some(fd)
            }
        }
        None
    }
    
    pub fn finger_up(&self, actions: &WidgetActions) -> Option<FingerUpEvent> {
        if let Some(item) = actions.find_single_action(self.widget_uid()) {
            if let ViewAction::FingerUp(fd) = item.action() {
                return Some(fd)
            }
        }
        None
    }
    
    pub fn finger_move(&self, actions: &WidgetActions) -> Option<FingerMoveEvent> {
        if let Some(item) = actions.find_single_action(self.widget_uid()) {
            if let ViewAction::FingerMove(fd) = item.action() {
                return Some(fd)
            }
        }
        None
    }
    
    pub fn key_down(&self, actions: &WidgetActions) -> Option<KeyEvent> {
        if let Some(item) = actions.find_single_action(self.widget_uid()) {
            if let ViewAction::KeyDown(fd) = item.action() {
                return Some(fd)
            }
        }
        None
    }
    
    pub fn key_up(&self, actions: &WidgetActions) -> Option<KeyEvent> {
        if let Some(item) = actions.find_single_action(self.widget_uid()) {
            if let ViewAction::KeyUp(fd) = item.action() {
                return Some(fd)
            }
        }
        None
    }
    
    pub fn cut_state(&self, cx: &mut Cx, state: &[LiveId; 2]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.animator_cut(cx, state);
        }
    }
    
    pub fn animator_play(&self, cx: &mut Cx, state: &[LiveId; 2]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.animator_play(cx, state);
        }
    }
    
    pub fn toggle_state(&self, cx: &mut Cx, is_state_1: bool, animate: Animate, state1: &[LiveId; 2], state2: &[LiveId; 2]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.animator_toggle(cx, is_state_1, animate, state1, state2);
        }
    }
    
    pub fn set_visible(&self, visible: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.visible = visible
        }
    }
    
    
    pub fn set_visible_and_redraw(&self, cx: &mut Cx, visible: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.visible = visible;
            inner.redraw(cx);
        }
    }
    
    pub fn visible(&self) -> bool {
        if let Some(inner) = self.borrow() {
            inner.visible
        }
        else {
            false
        }
    }
    
    pub fn set_texture(&self, slot: usize, texture: &Texture) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.draw_bg.set_texture(slot, texture);
        }
    }
    
    pub fn set_uniform(&self, cx: &Cx, uniform: &[LiveId], value: &[f32]) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.draw_bg.set_uniform(cx, uniform, value);
        }
    }
    
    pub fn set_scroll_pos(&self, cx: &mut Cx, v: DVec2) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.set_scroll_pos(cx, v)
        }
    }
    
    pub fn area(&self) -> Area {
        if let Some(inner) = self.borrow_mut() {
            inner.area
        }
        else {
            Area::Empty
        }
    }
    
    pub fn child_count(&self) -> usize {
        if let Some(inner) = self.borrow_mut() {
            inner.draw_order.len()
        }
        else {
            0
        }
    }

    pub fn append_child(&self, cx: &mut Cx, id: LiveId, ptr: Option<LivePtr>) -> Option<WidgetRef> {
        if let Some(mut inner) = self.borrow_mut() {
            inner.draw_order.push(id);
            let widget = inner.children.get_or_insert(cx, id, | cx | {
                WidgetRef::new_from_ptr(cx, ptr)
            });

            Some(widget.clone())
        }
        else {
            None
        }
    }

    pub fn clear_children(&self) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.draw_order.clear();
            inner.children.clear();
        }
    }
}

impl ViewSet {
    
    pub fn cut_state(&mut self, cx: &mut Cx, state: &[LiveId; 2]) {
        for item in self.iter() {
            item.cut_state(cx, state)
        }
    }
    
    pub fn animator_play(&mut self, cx: &mut Cx, state: &[LiveId; 2]) {
        for item in self.iter() {
            item.animator_play(cx, state);
        }
    }
    
    pub fn toggle_state(&mut self, cx: &mut Cx, is_state_1: bool, animate: Animate, state1: &[LiveId; 2], state2: &[LiveId; 2]) {
        for item in self.iter() {
            item.toggle_state(cx, is_state_1, animate, state1, state2);
        }
    }
    
    pub fn set_visible(&self, visible: bool) {
        for item in self.iter() {
            item.set_visible(visible)
        }
    }
    
    pub fn set_texture(&self, slot: usize, texture: &Texture) {
        for item in self.iter() {
            item.set_texture(slot, texture)
        }
    }
    
    pub fn set_uniform(&self, cx: &Cx, uniform: &[LiveId], value: &[f32]) {
        for item in self.iter() {
            item.set_uniform(cx, uniform, value)
        }
    }
    
    pub fn redraw(&self, cx: &mut Cx) {
        for item in self.iter() {
            item.redraw(cx);
        }
    }
    
    pub fn finger_down(&self, actions: &WidgetActions) -> Option<FingerDownEvent> {
        for item in self.iter() {
            if let Some(e) = item.finger_down(actions) {
                return Some(e)
            }
        }
        None
    }
    
    pub fn finger_up(&self, actions: &WidgetActions) -> Option<FingerUpEvent> {
        for item in self.iter() {
            if let Some(e) = item.finger_up(actions) {
                return Some(e)
            }
        }
        None
    }
    
    
    pub fn finger_move(&self, actions: &WidgetActions) -> Option<FingerMoveEvent> {
        for item in self.iter() {
            if let Some(e) = item.finger_move(actions) {
                return Some(e)
            }
        }
        None
    }
    
    pub fn key_down(&self, actions: &WidgetActions) -> Option<KeyEvent> {
        for item in self.iter() {
            if let Some(e) = item.key_down(actions) {
                return Some(e)
            }
        }
        None
    }
    
    pub fn key_up(&self, actions: &WidgetActions) -> Option<KeyEvent> {
        for item in self.iter() {
            if let Some(e) = item.key_up(actions) {
                return Some(e)
            }
        }
        None
    }
}

impl Widget for View {
    fn handle_widget_event_with(
        &mut self,
        cx: &mut Cx,
        event: &Event,
        dispatch_action: &mut dyn FnMut(&mut Cx, WidgetActionItem)
    ) {
        let uid = self.widget_uid();
        if self.animator_handle_event(cx, event).must_redraw() {
            self.redraw(cx);
        }
        
        if self.block_signal_event {
            if let Event::Signal = event {
                return
            }
        }
        if let Some(scroll_bars) = &mut self.scroll_bars_obj {
            let mut redraw = false;
            scroll_bars.handle_main_event(cx, event, &mut | _, _ | {
                // lets invalidate all children
                redraw = true;
            });
            if redraw {
                cx.redraw_area_and_children(self.area);
            }
        }
        
        match &self.event_order {
            EventOrder::Up => {
                for id in self.draw_order.iter().rev() {
                    if let Some(child) = self.children.get_mut(id) {
                        if child.is_visible() || !event.requires_visibility() {
                            child.handle_widget_event_with(cx, event, dispatch_action);
                        }
                    }
                }
            }
            EventOrder::Down => {
                for id in self.draw_order.iter() {
                    if let Some(child) = self.children.get_mut(id) {
                        if child.is_visible() || !event.requires_visibility() {
                            child.handle_widget_event_with(cx, event, dispatch_action);
                        }
                    }
                }
            }
            EventOrder::List(list) => {
                for id in list {
                    if let Some(child) = self.children.get_mut(id) {
                        if child.is_visible() || !event.requires_visibility() {
                            child.handle_widget_event_with(cx, event, dispatch_action);
                        }
                    }
                }
            }
        }
        
        
        if self.visible && self.cursor.is_some() || self.animator.live_ptr.is_some() {
            match event.hits(cx, self.area()) {
                Hit::FingerDown(e) => {
                    if self.grab_key_focus {
                        cx.set_key_focus(self.area());
                    }
                    dispatch_action(cx, ViewAction::FingerDown(e).into_action(uid));
                    if self.animator.live_ptr.is_some() {
                        self.animator_play(cx, id!(down.on));
                    }
                }
                Hit::FingerMove(e) => {
                    dispatch_action(cx, ViewAction::FingerMove(e).into_action(uid))
                }
                Hit::FingerUp(e) => {
                    dispatch_action(cx, ViewAction::FingerUp(e).into_action(uid));
                    if self.animator.live_ptr.is_some() {
                        self.animator_play(cx, id!(down.off));
                    }
                }
                Hit::FingerHoverIn(_) => {
                    if let Some(cursor) = &self.cursor {
                        cx.set_cursor(*cursor);
                    }
                    if self.animator.live_ptr.is_some() {
                        self.animator_play(cx, id!(hover.on));
                    }
                }
                Hit::FingerHoverOut(_) => {
                    if self.animator.live_ptr.is_some() {
                        self.animator_play(cx, id!(hover.off));
                    }
                }
                Hit::KeyDown(e) => {
                    dispatch_action(cx, ViewAction::KeyDown(e).into_action(uid))
                }
                Hit::KeyUp(e) => {
                    dispatch_action(cx, ViewAction::KeyUp(e).into_action(uid))
                }
                _ => ()
            }
        }
        
        if let Some(scroll_bars) = &mut self.scroll_bars_obj {
            scroll_bars.handle_scroll_event(cx, event, &mut | _, _ | {});
        }
    }
    
    fn is_visible(&self) -> bool {
        self.visible
    }
    
    fn walk(&mut self, _cx: &mut Cx) -> Walk {
        self.walk
    }
    
    fn draw_walk_widget(&mut self, cx: &mut Cx2d, walk: Walk) -> WidgetDraw {
        self.draw_walk(cx, walk)
    }
    
    fn redraw(&mut self, cx: &mut Cx) {
        self.area.redraw(cx);
        for child in self.children.values_mut() {
            child.redraw(cx);
        }
    }
    
    fn find_widgets(&mut self, path: &[LiveId], cached: WidgetCache, results: &mut WidgetSet) {
        match cached {
            WidgetCache::Yes | WidgetCache::Clear => {
                if let WidgetCache::Clear = cached {
                    self.find_cache.clear();
                }
                let mut hash = 0u64;
                for i in 0..path.len() {
                    hash ^= path[i].0
                }
                if let Some(widget_set) = self.find_cache.get(&hash) {
                    results.extend_from_set(widget_set);
                    return
                }
                let mut local_results = WidgetSet::empty();
                if let Some(child) = self.children.get_mut(&path[0]) {
                    if path.len()>1 {
                        child.find_widgets(&path[1..], WidgetCache::No, &mut local_results);
                    }
                    else {
                        local_results.push(child.clone());
                    }
                }
                for child in self.children.values_mut() {
                    child.find_widgets(path, WidgetCache::No, &mut local_results);
                }
                if !local_results.is_empty() {
                    results.extend_from_set(&local_results);
                }
                self.find_cache.insert(hash, local_results);
            }
            WidgetCache::No => {
                if let Some(child) = self.children.get_mut(&path[0]) {
                    if path.len()>1 {
                        child.find_widgets(&path[1..], WidgetCache::No, results);
                    }
                    else {
                        results.push(child.clone());
                    }
                }
                for child in self.children.values_mut() {
                    child.find_widgets(path, WidgetCache::No, results);
                }
            }
        }
    }
}

#[derive(Clone)]
enum DrawState {
    Drawing(usize, bool),
    DeferWalk(usize)
}

impl View {
    
    pub fn set_scroll_pos(&mut self, cx: &mut Cx, v: DVec2) {
        if let Some(scroll_bars) = &mut self.scroll_bars_obj {
            scroll_bars.set_scroll_pos(cx, v);
        }
        else {
            self.layout.scroll = v;
        }
    }
    
    pub fn area(&self) -> Area {
        self.area
    }
    
    pub fn walk_from_previous_size(&self, walk: Walk) -> Walk {
        let view_size = self.view_size.unwrap_or(DVec2::default());
        Walk {
            abs_pos: walk.abs_pos,
            width: if walk.width.is_fill() {walk.width}else {Size::Fixed(view_size.x)},
            height: if walk.height.is_fill() {walk.height}else {Size::Fixed(view_size.y)},
            margin: walk.margin
        }
    }
    
    pub fn draw_walk(&mut self, cx: &mut Cx2d, walk: Walk) -> WidgetDraw {
        // the beginning state
        if self.draw_state.begin(cx, DrawState::Drawing(0, false)) {
            if !self.visible {
                self.draw_state.end();
                return WidgetDraw::done()
            }
            
            self.defer_walks.clear();
            
            match self.optimize {
                ViewOptimize::Texture => {
                    let walk = self.walk_from_previous_size(walk);
                    if !cx.will_redraw(self.draw_list.as_mut().unwrap(), walk) {
                        if let Some(texture_cache) = &self.texture_cache {
                            self.draw_bg.draw_vars.set_texture(0, &texture_cache.color_texture);
                            let mut rect = cx.walk_turtle_with_area(&mut self.area, walk);
                            rect.size *= 2.0 / self.dpi_factor.unwrap_or(1.0);
                            self.draw_bg.draw_abs(cx, rect);
                            self.area = self.draw_bg.area();
                            cx.set_pass_scaled_area(&texture_cache.pass, self.area, 2.0 / self.dpi_factor.unwrap_or(1.0));
                        }
                        return WidgetDraw::done()
                    }
                    // lets start a pass
                    if self.texture_cache.is_none() {
                        self.texture_cache = Some(ViewTextureCache {
                            pass: Pass::new(cx),
                            _depth_texture: Texture::new(cx),
                            color_texture: Texture::new(cx)
                        });
                        let texture_cache = self.texture_cache.as_mut().unwrap();
                        //cache.pass.set_depth_texture(cx, &cache.depth_texture, PassClearDepth::ClearWith(1.0));
                        texture_cache.pass.add_color_texture(cx, &texture_cache.color_texture, PassClearColor::ClearWith(vec4(0.0, 0.0, 0.0, 0.0)));
                    }
                    let texture_cache = self.texture_cache.as_mut().unwrap();
                    cx.make_child_pass(&texture_cache.pass);
                    cx.begin_pass(&texture_cache.pass, self.dpi_factor);
                    self.draw_list.as_mut().unwrap().begin_always(cx)
                }
                ViewOptimize::DrawList => {
                    let walk = self.walk_from_previous_size(walk);
                    if self.draw_list.as_mut().unwrap().begin(cx, walk).is_not_redrawing() {
                        cx.walk_turtle_with_area(&mut self.area, walk);
                        return WidgetDraw::done()
                    }
                }
                _ => ()
            }
            
            
            // ok so.. we have to keep calling draw till we return LiveId(0)
            let scroll = if let Some(scroll_bars) = &mut self.scroll_bars_obj {
                scroll_bars.begin_nav_area(cx);
                scroll_bars.get_scroll_pos()
            }
            else {
                self.layout.scroll
            };
            
            if self.show_bg {
                /*if let Some(image_texture) = &self.image_texture {
                    self.draw_bg.draw_vars.set_texture(0, image_texture);
                }*/
                self.draw_bg.begin(cx, walk, self.layout.with_scroll(scroll)); //.with_scale(2.0 / self.dpi_factor.unwrap_or(2.0)));
            }
            else {
                cx.begin_turtle(walk, self.layout.with_scroll(scroll)); //.with_scale(2.0 / self.dpi_factor.unwrap_or(2.0)));
            }
        }
        
        while let Some(DrawState::Drawing(step, resume)) = self.draw_state.get() {
            if step < self.draw_order.len() {
                let id = self.draw_order[step];
                if let Some(child) = self.children.get_mut(&id) {
                    if child.is_visible() {
                        let walk = child.walk(cx);
                        if resume {
                            child.draw_walk_widget(cx, walk) ?;
                        }
                        else if let Some(fw) = cx.defer_walk(walk) {
                            self.defer_walks.push((id, fw));
                        }
                        else {
                            self.draw_state.set(DrawState::Drawing(step, true));
                            child.draw_walk_widget(cx, walk) ?;
                        }
                    }
                }
                self.draw_state.set(DrawState::Drawing(step + 1, false));
            }
            else {
                self.draw_state.set(DrawState::DeferWalk(0));
            }
        }
        
        while let Some(DrawState::DeferWalk(step)) = self.draw_state.get() {
            if step < self.defer_walks.len() {
                let (id, dw) = &mut self.defer_walks[step];
                if let Some(child) = self.children.get_mut(&id) {
                    let walk = dw.resolve(cx);
                    child.draw_walk_widget(cx, walk) ?;
                }
                self.draw_state.set(DrawState::DeferWalk(step + 1));
            }
            else {
                if let Some(scroll_bars) = &mut self.scroll_bars_obj {
                    scroll_bars.draw_scroll_bars(cx);
                };
                
                if self.show_bg {
                    if self.optimize.is_texture() {
                        panic!("dont use show_bg and texture cazching at the same time");
                    }
                    self.draw_bg.end(cx);
                    self.area = self.draw_bg.area();
                }
                else {
                    cx.end_turtle_with_area(&mut self.area);
                };
                
                if let Some(scroll_bars) = &mut self.scroll_bars_obj {
                    scroll_bars.set_area(self.area);
                    scroll_bars.end_nav_area(cx);
                };
                
                if self.optimize.needs_draw_list() {
                    let rect = self.area.get_rect(cx);
                    self.view_size = Some(rect.size);
                    self.draw_list.as_mut().unwrap().end(cx);
                    
                    if self.optimize.is_texture() {
                        let texture_cache = self.texture_cache.as_mut().unwrap();
                        cx.end_pass(&texture_cache.pass);
                        /*if cache.pass.id_equals(4){
                            self.draw_bg.draw_vars.set_uniform(cx, id!(marked),&[1.0]);
                        }
                        else{
                            self.draw_bg.draw_vars.set_uniform(cx, id!(marked),&[0.0]);
                        }*/
                        self.draw_bg.draw_vars.set_texture(0, &texture_cache.color_texture);
                        self.draw_bg.draw_abs(cx, rect);
                        let area = self.draw_bg.area();
                        let texture_cache = self.texture_cache.as_mut().unwrap();
                        cx.set_pass_scaled_area(&texture_cache.pass, area, 2.0 / self.dpi_factor.unwrap_or(1.0));
                    }
                }
                self.draw_state.end();
            }
        }
        WidgetDraw::done()
    }
    
    pub fn child_count(&self) -> usize {
        self.draw_order.len()
    }
}

