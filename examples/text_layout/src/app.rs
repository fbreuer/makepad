use makepad_widgets::*;
use makepad_rustybuzz::{shape, Face, GlyphBuffer, UnicodeBuffer, Feature, ttf_parser};
use makepad_widgets::font_atlas::{CxFont, CxFontsAtlas, CxFontsAtlasRc};
use makepad_draw::owned_font_face::OwnedFace;

live_design!{
    import makepad_widgets::base::*;
    import makepad_widgets::theme_desktop_dark::*; 
    
    App = {{App}} {
        ui: <Root>{
            main_window = <Window>{
                show_bg: true
                width: Fill,
                height: Fill
                
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        // test
                        return mix(#7, #3, self.pos.y);
                    }
                }
                
                body = <ScrollXYView>{
                    flow: Down,
                    spacing:10,
                    align: {
                        x: 0.5,
                        y: 0.5
                    },
                    button1 = <Button> {
                        text: "Hello world"
                        draw_text:{color:#f00}
                    }
                    input1 = <TextInput> {
                        width: 100, height: 30
                        text: "Click to count "
                    }
                    label1 = <Label> {
                        draw_text: {
                            color: #f
                        },
                        text: "Counter: 0"
                    }
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
        //println!("{}", std::mem::size_of::<LiveNode2>());
        /*makepad_draw::live_design(cx);
        makepad_widgets::base::live_design(cx);
        makepad_widgets::theme_desktop_dark::live_design(cx);
        makepad_widgets::label::live_design(cx);
        makepad_widgets::view::live_design(cx);
        makepad_widgets::button::live_design(cx);
        makepad_widgets::window::live_design(cx);
        makepad_widgets::scroll_bar::live_design(cx);
        makepad_widgets::scroll_bars::live_design(cx);
        makepad_widgets::root::live_design(cx);*/
        crate::makepad_widgets::live_design(cx);
    }
}

impl MatchEvent for App{
    fn handle_actions(&mut self, cx: &mut Cx, actions:&Actions){
        if self.ui.button(id!(button1)).clicked(&actions) {
            log!("BUTTON jk {}", self.counter); 
            self.counter += 1;
            let label = self.ui.label(id!(label1));
            label.set_text_and_redraw(cx,&format!("Counter: {}", self.counter));
            //log!("TOTAL : {}",TrackingHeap.total());

            dbg!("got here");

            Cx2d::lazy_construct_font_atlas(cx);
            let atlas_ref: CxFontsAtlasRc = cx.get_global::<CxFontsAtlasRc>().clone();
            let mut atlas = atlas_ref.0.borrow_mut();
            // loading a custom font like this does not work, it does not get populated correctly for some reason and the lookup fails...
            let font_id: usize = atlas.get_font_by_path(cx, "C:\\Users\\felix\\Dropbox\\Dev\\Macro\\macro-notes\\2023-11-29 Berkeley Font Patch\\out\\BerkeleyMonoNerdFont-Regular.otf");
            // ... however when I hard-code the 0 index, it get the default font, which works just fine
            let font_option: Option<&CxFont> = atlas.fonts[0].as_ref();
            let font: &CxFont = match font_option {
                Some(x) => x,
                None => todo!(),
            };
            // owned_face is actually private and I should not have access to this at all
            let owned_face: &OwnedFace = &font.owned_font_face;

            let mut unicode_buffer: UnicodeBuffer = UnicodeBuffer::new(); // TODO
            unicode_buffer.push_str("hello->==");
            let glyph_buffer: GlyphBuffer = owned_face.with_ref( | face | makepad_rustybuzz::shape(face, &[], unicode_buffer));

            dbg!(glyph_buffer);
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
} 

/*

// This is our custom allocator!
use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicU64, Ordering},
};

pub struct TrackingHeapWrap{
    count: AtomicU64,
    total: AtomicU64,
}

impl TrackingHeapWrap {
    // A const initializer that starts the count at 0.
    pub const fn new() -> Self {
        Self{
            count: AtomicU64::new(0),
            total: AtomicU64::new(0)
        }
    }
    
    // Returns the current count.
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }
    
    pub fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }
}

unsafe impl GlobalAlloc for TrackingHeapWrap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Pass everything to System.
        self.count.fetch_add(1, Ordering::Relaxed); 
        self.total.fetch_add(layout.size() as u64, Ordering::Relaxed);
        System.alloc(layout)
    }
        
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.count.fetch_sub(1, Ordering::Relaxed); 
        self.total.fetch_sub(layout.size() as u64, Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }
}

// Register our custom allocator.
#[global_allocator]
static TrackingHeap: TrackingHeapWrap = TrackingHeapWrap::new();*/