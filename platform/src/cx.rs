use {
    std::{
        collections::{
            HashMap,
            HashSet,
        },
        any::{Any, TypeId},
        rc::Rc,
        rc::Weak,
        cell::RefCell,
    },
    crate::{
        makepad_live_compiler::{
            LiveRegistry
        },
        makepad_shader_compiler::{
            ShaderRegistry
        },
        cx_draw_shaders::{
            CxDrawShaders
        },
        platform::{
            CxPlatform,
        },
        debug::Debug,
        event::{
            DrawEvent,
            CxFingers,
            CxFingerDrag,
            Event,
            Signal,
            Trigger,
            CxKeyboard,
            NextFrame,
        },
        menu::{
            CxCommandSetting,
            MenuCommand
        },
        cx_api::{CxPlatformOp},
        area::{
            Area,
        },
        gpu_info::GpuInfo,
        window::{
            CxWindowPool,
        },
        draw_list::{
            CxDrawListPool
        },
        pass::{
            CxPassPool,
        },
        texture::{
            CxTexturePool
        },
        geometry::{
            Geometry,
            CxGeometryPool,
            GeometryFingerprint
        },
    }
};

pub use makepad_shader_compiler::makepad_derive_live::*;
pub use makepad_shader_compiler::makepad_math::*;

pub struct Cx {
    pub (crate) platform_type: PlatformType,
    pub (crate) gpu_info: GpuInfo,
    pub (crate) cpu_cores: usize,
    
    pub windows: CxWindowPool,
    pub passes: CxPassPool,
    pub draw_lists: CxDrawListPool,
    pub (crate) textures: CxTexturePool,
    pub (crate) geometries: CxGeometryPool,
    
    pub (crate) geometries_refs: HashMap<GeometryFingerprint, Weak<Geometry >>,
    
    pub draw_shaders: CxDrawShaders,
    
    pub (crate) new_draw_event: DrawEvent,
    
    pub redraw_id: u64,
    pub (crate) repaint_id: u64,
    pub (crate) event_id: u64,
    pub (crate) timer_id: u64,
    pub (crate) next_frame_id: u64,
    
    #[allow(dead_code)]
    pub (crate) web_socket_id: u64,
    
    pub (crate) keyboard: CxKeyboard,
    pub (crate) fingers: CxFingers,
    pub (crate) finger_drag: CxFingerDrag,
    
    pub (crate) platform_ops: Vec<CxPlatformOp>,
    
    pub (crate) new_next_frames: HashSet<NextFrame>,
    
    pub (crate) dependencies: HashMap<String, CxDependency>,
    
    pub (crate) signals: HashSet<Signal>,
    pub (crate) triggers: HashMap<Area, HashSet<Trigger >>,
    
    pub live_registry: Rc<RefCell<LiveRegistry >>,
    pub shader_registry: ShaderRegistry,
    
    #[allow(dead_code)]
    pub (crate) command_settings: HashMap<MenuCommand, CxCommandSetting>,
    
    pub (crate) platform: CxPlatform,
    // (cratethis cuts the compiletime of an end-user application in half
    pub (crate) event_handler: Option<Box<dyn FnMut(&mut Cx, &Event)>>,

    pub (crate) globals: Vec<(TypeId, Box<dyn Any>)>,

    pub debug:Debug,

}

pub struct CxDependency {
    pub data: Option<Result<Vec<u8>, String >>
}


#[derive(Clone)]
pub enum PlatformType {
    Unknown,
    MsWindows,
    OSX,
    Linux {custom_window_chrome: bool},
    WebBrowser {protocol: String, host: String, hostname: String, pathname: String, search: String, hash: String}
}

impl PlatformType {
    pub fn is_desktop(&self) -> bool {
        match self {
            PlatformType::Unknown => true,
            PlatformType::MsWindows => true,
            PlatformType::OSX => true,
            PlatformType::Linux {..} => true,
            PlatformType::WebBrowser {..} => false
        }
    }
}

impl Cx {
    pub fn new(event_handler:Box<dyn FnMut(&mut Cx, &Event)>) -> Self {
        // the null texture
        /*let mut textures = CxTexturePool::default();
        textures.alloc_new(CxTexture {
            desc: TextureDesc {
                format: TextureFormat::ImageBGRA,
                width: Some(4),
                height: Some(4),
                multisample: None
            },
            image_u32: vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            //image_f32: Vec::new(),
            update_image: true,
            platform: CxPlatformTexture::default()
        });*/
        
        Self {
            cpu_cores: 8,
            
            platform_type: PlatformType::Unknown,
            gpu_info: GpuInfo::default(),
            
            windows: Default::default(),
            passes: Default::default(),
            draw_lists: Default::default(),
            geometries: Default::default(),
            textures:CxTexturePool::default(),
            
            geometries_refs: HashMap::new(),
            
            draw_shaders: CxDrawShaders::default(),
            
            new_draw_event: DrawEvent::default(),
            
            redraw_id: 1,
            event_id: 1,
            repaint_id: 1,
            timer_id: 1,
            next_frame_id: 1,
            web_socket_id: 1,
            
            keyboard: CxKeyboard::default(),
            fingers: CxFingers::default(),
            finger_drag: CxFingerDrag::default(),
            
            platform_ops: Vec::new(),
            
            
            new_next_frames: HashSet::new(),
            
            dependencies: HashMap::new(),
            
            signals: HashSet::new(),
            triggers: HashMap::new(),
            
            live_registry: Rc::new(RefCell::new(LiveRegistry::default())),
            shader_registry: ShaderRegistry::new(),
            
            command_settings: HashMap::new(),
            
            platform: CxPlatform {..Default::default()},
            
            event_handler:Some(event_handler),
            
            debug: Default::default(),

            globals: Vec::new(),
        }
    }
}


