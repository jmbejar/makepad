use {
    std::rc::Rc, 
    std::cell::RefCell,
    std::collections::{HashMap,hash_map},
    crate::{
        makepad_code_editor::{Session,Document},
        makepad_code_editor::code_editor::*,
        makepad_platform::*,
        makepad_draw::*,
        makepad_widgets::*,
        makepad_widgets::file_tree::*,
        makepad_widgets::dock::*,
        file_client::file_system::*,
        run_view::*,
        build::{
            build_manager::{
                BuildManager,
                BuildManagerAction
            },
        },
    },
};

live_design!{
    import makepad_draw::shader::std::*;
    import makepad_widgets::theme::*;
    import makepad_widgets::frame::*;
    import makepad_widgets::file_tree::FileTree;
    import makepad_widgets::dock::*;
    import makepad_widgets::desktop_window::DesktopWindow;
    import makepad_code_editor::code_editor::CodeEditor;
    
    import makepad_studio::run_view::RunView;
    import makepad_studio::build::build_manager::LogList;
    
    App = {{App}} {
        ui: <DesktopWindow> {
            caption_bar = {visible: true, caption_label = {label = {label: "Makepad Studio"}}},
            dock = <Dock> {
                walk: {height: Fill, width: Fill}
                
                root = Splitter {
                    axis: Horizontal,
                    align: FromA(200.0),
                    a: file_tree,
                    b: split1
                }
                
                split1 = Splitter {
                    axis: Vertical,
                    align: FromB(200.0),
                    a: split2,
                    b: log_list
                }
                
                split2 = Splitter {
                    axis: Horizontal,
                    align: FromB(400.0),
                    a: open_files,
                    b: run_views
                }
                
                open_files = Tabs {
                    tabs: [file1, file2, file3],
                    selected: 0
                }
                
                run_views = Tabs {
                    tabs: [run_view],
                    selected: 0
                }
                
                file1 = Tab {
                    name: "Empty1"
                    kind: CodeEditor
                }
                
                file2 = Tab {
                    name: "File2"
                    kind: Empty2
                }
                
                file3 = Tab {
                    name: "File3"
                    kind: Empty3
                }
                
                file_tree = Tab {
                    name: "FileTree",
                    kind: FileTree
                }
                
                log_list = Tab {
                    name: "LogList",
                    kind: LogList
                }
                
                run_view = Tab {
                    name: "Run",
                    kind: RunView
                }
                CodeEditor = <CodeEditor>{}
                Empty1 = <Rect> {draw_bg: {color: #533}}
                Empty2 = <Rect> {draw_bg: {color: #353}}
                Empty3 = <Rect> {draw_bg: {color: #335}}
                Empty4 = <Rect> {draw_bg: {color: #535}}
                RunView = <RunView> {}
                FileTree = <FileTree> {}
                LogList = <LogList> {}
                //LogList = <LogList>{}
            }
        }
    }
}

#[derive(Live)]
pub struct App {
    #[live] ui: WidgetRef,
    #[live] build_manager: BuildManager,
    #[rust] file_system: FileSystem,
    #[rust] sessions: HashMap<LiveId,Session>,
}

impl LiveHook for App {
    fn before_live_design(cx: &mut Cx) {
        crate::makepad_widgets::live_design(cx);
        crate::makepad_code_editor::live_design(cx);
        crate::build::build_manager::live_design(cx);
        crate::run_view::live_design(cx);
    }
    
    fn after_new_from_doc(&mut self, cx: &mut Cx) {
        self.file_system.init(cx);
        self.build_manager.init(cx);
    }
}

app_main!(App);

impl App {
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        let dock = self.ui.get_dock(id!(dock));
        let file_tree = self.ui.get_file_tree(id!(file_tree));
        let run_view = self.ui.get_run_view(id!(run_view));
        let log_list = self.ui.get_list_view(id!(log_list));
        
        if let Event::Draw(event) = event {
            let cx = &mut Cx2d::new(cx, event);
            while let Some(next) = self.ui.draw_widget(cx).hook_widget() {
                
                if let Some(mut file_tree) = file_tree.has_widget(&next).borrow_mut() {
                    file_tree.set_folder_is_open(cx, live_id!(root).into(), true, Animate::No);
                    self.file_system.draw_file_node(
                        cx,
                        live_id!(root).into(),
                        &mut *file_tree
                    );
                }
                else if let Some(mut run_view) = run_view.has_widget(&next).borrow_mut() {
                    run_view.draw(cx, &self.build_manager);
                }
                else if let Some(mut list_view) = log_list.has_widget(&next).borrow_mut() {
                    self.build_manager.draw_log_list(cx, &mut *list_view);
                }
                else if let Some(mut code_editor) = next.as_code_editor().borrow_mut(){
                    // lets fetch a session
                    let current_id = dock.get_drawing_item_id().unwrap();
                    let session = match self.sessions.entry(current_id){
                        hash_map::Entry::Occupied(o) => o.into_mut(),
                        hash_map::Entry::Vacant(v) => v.insert(Session::new(Rc::new(RefCell::new(Document::new(
                            include_str!("app.rs").into(),
                        )))))
                    };
                    code_editor.draw(cx, session)
                }
            }
            return
        }
        
        self.file_system.handle_event(cx, event, &self.ui);
        
        if let Some(mut run_view) = run_view.borrow_mut(){
            run_view.handle_event(cx, event, &mut self.build_manager);
        }
        
        // lets iterate over the editors and handle events
        for (item_id,item) in dock.borrow_mut().unwrap().items().iter(){
            if let Some(mut code_editor) = item.as_code_editor().borrow_mut(){
                if let Some(session) = self.sessions.get_mut(&item_id.id){
                     code_editor.handle_event(cx, event, session);
                 }
             }
        }
        
        for action in self.build_manager.handle_event(cx, event) {
            match action {
                BuildManagerAction::RedrawLog => {
                    log_list.redraw(cx);
                }
                BuildManagerAction::StdinToHost{cmd_id, msg}=>if let Some(mut run_view) = run_view.borrow_mut(){
                    run_view.handle_stdin_to_host(cx, cmd_id, msg, &mut self.build_manager);
                }
                _ => ()
            }
        }
        
        let actions = self.ui.handle_widget_event(cx, event);
        
        // dock drag drop and tabs
        
        if let Some(tab_id) = dock.clicked_tab_close(&actions) {
            dock.close_tab(cx, tab_id);
        }
        
        if let Some(tab_id) = dock.should_tab_start_drag(&actions) {
            dock.tab_start_drag(cx, tab_id, DragItem::FilePath {
                path: "".to_string(), //String::from("file://") + &*path.into_unix_string().to_string_lossy(),
                internal_id: Some(tab_id)
            });
        }
        
        if let Some(drag) = dock.should_accept_drag(&actions) {
            if drag.items.len() == 1 {
                if drag.modifiers.logo {
                    dock.accept_drag(cx, drag, DragResponse::Copy);
                }
                else {
                    dock.accept_drag(cx, drag, DragResponse::Move);
                }
            }
        }
        
        if let Some(drop) = dock.has_drop(&actions) {
            if let DragItem::FilePath {path, internal_id} = &drop.items[0] {
                if let Some(internal_id) = internal_id { // from inside the dock
                    if drop.modifiers.logo {
                        dock.drop_clone(cx, drop.abs, *internal_id, LiveId::unique());
                    }
                    else {
                        dock.drop_move(cx, drop.abs, *internal_id);
                    }
                }
                else { // external file, we have to create a new tab
                    dock.drop_create(cx, drop.abs, LiveId::unique(), live_id!(Empty4), path.clone())
                }
            }
        }
        
        if let Some(file_id) = file_tree.should_file_start_drag(&actions) {
            let path = self.file_system.file_nodes.get(&file_id).unwrap().name.clone();
            file_tree.file_start_drag(cx, file_id, DragItem::FilePath {
                path,
                internal_id: None
            });
        }
        
        if let Some(file_id) = file_tree.file_clicked(&actions) {
            let path = self.file_system.file_nodes.get(&file_id).unwrap().name.clone();
            // lets add a file tab 'somewhere'
            dock.create_tab(cx, live_id!(content2), LiveId::unique(), live_id!(Empty4), path);
        }
    }
}
