const SOURCE:&'static str = r#"
        DrawQuad: Shader{
            uniform t:float
            
            fn closure_inner(self, t:float, b:fn(v2:float)->float){
                b(3. + t);
            }
            
            fn closure_test(self, x1:float, y:fn(v2:float)->float, z:fn(v2:float)->float){
                y(1.+x1);
                z(2.+x1);
                self.closure_inner(1.0, |w|w + x1);
            }
            
            fn pixel(self)->vec4{
                let i = 1.0;
                let j = 2.0;
                let t = |x| x+j;
                let j = 2.0;
                self.closure_test(1.0, |x| x+i+self.t+j, t);
                //self.closure_test(2.0, t);
                return #f00;
            }
            
            fn vertex(self)->vec4{
                return vec4(1.0);
            }
        }
/*
        ViewShader: Shader{
            uniform camera_projection: mat4 in pass;
            uniform draw_scroll: vec4 in draw;
            //instance y: float
        }
        
        // what does this thing factory?
        DrawQuad: ViewShader{
            // these point to things in Rust
            draw_input self::DrawQuad;
            default_geometry makepad_render::shader_std::quad_2d;
            
            geometry geom: vec2;
            //instance x: float
            //instance y: float
            uniform z: float
            varying w: float

            BlaComp:Component{
                fn blup()->int{return 0;}
            }
            
            const CV:float = 1.0;
            bla: 1.0,
            
            fn closure_test(self, x:float, y:fn(v1:float, v2:float)->float){
                y(1.0,2.0);
            }
            
            MyStruct2:Struct{
                field b:float
                fn blip(self){}
            }
            
            MyStruct:Struct{
                field x:float
                field y:float
                field z:float
                field bb: MyStruct2
                fn blop(self){}
                fn bla()->Self{
                    let t = BlaComp::blup();
                    let v = Self{x:1.0,y:2.0,z:3.0,bb:MyStruct2{b:1.0}};
                    v.x = CV;
                    v.y = 2.0;
                    v.z = bla;
                    v.bb.blip();
                    v.blop();
                    return v;
                }
            }
            
            fn other(self, x:float)->vec4{
                return vec4(self.w+2.0);
            }
            
            fn pixel(self)->vec4{
                self.closure_test(1.0, |x, y| x+y);
                
                let y = MyStruct{x:1.0,y:2.0,z:3.0,bb:MyStruct2{b:1.0}};
                let x = MyStruct::bla();
                self.other(1.0 + self.duni + self.dinst);
                //let w = self.z;
                return #f00;
            }
            
            fn vertex(self)->vec4{
                self.w = 1.0;
                return vec4(1.0);
            }
        }
        */
    "#;
    
use makepad_live_parser::*;
use makepad_shader_compiler::shaderregistry::ShaderRegistry;
use makepad_shader_compiler::shaderregistry::DrawShaderInput;
use makepad_shader_compiler::shaderast::TyLit;
/*
#[derive(Clone, Debug)]
struct DrawQuad{
}

impl DeLive for DrawQuad{
    fn de_live(lr: &LiveRegistry, file: usize, level: usize, index: usize) -> Result<Self,
    DeLiveErr>{
        // ok lets parse this
        
        
        return Ok(DrawQuad{})
    }
}

struct MyShaderFactory {}
impl LiveFactoryTest for MyShaderFactory {
    fn de_live_any(&self, lr: &LiveRegistry, file: usize, level: usize, index: usize) -> Result<Box<dyn Any>,
    DeLiveErr> {
        // lets give the shader compiler out component.
        // alright so.. lets parse the shader
        let mv = DrawQuad::de_live(lr, file, level, index) ?;
        Ok(Box::new(mv))
    }
}
*/
fn main() {
    //println!("{}", std::mem::size_of::<LiveNode>());
    // ok lets do a deserialize
    //let mut lr = LiveFactoriesTest::default();
    let mut sr = ShaderRegistry::new();
    
    match sr.live_registry.parse_live_file("test.live", id_check!(main), id_check!(test), SOURCE.to_string()) {
        Err(why) => panic!("Couldnt parse file {}", why),
        _ => ()
    }
    
    let mut errors = Vec::new();
    sr.live_registry.expand_all_documents(&mut errors);
    
    //println!("{}", lr.registry.expanded[0]);
    
    for msg in errors {
        println!("{}\n", msg.to_live_file_error("", SOURCE));
    }
    
    let mut di = DrawShaderInput::default();
    di.add_uniform("duni", TyLit::Float.to_ty_expr());
    di.add_instance("dinst", TyLit::Float.to_ty_expr());
    sr.register_draw_input("main::test", "DrawQuad", di);
    
    // lets just call the shader compiler on this thing
    let result = sr.analyse_draw_shader(id!(main), id!(test), &[id!(DrawQuad)]);
    match result{
        Err(e)=>{
            println!("Error {}", e.to_live_file_error("", SOURCE));
        }
        Ok(_)=>{
            println!("OK!");
        }
    }
    // ok the shader is analysed.
    // now we will generate the glsl shader.
    let result = sr.generate_glsl_shader(id!(main), id!(test), &[id!(DrawQuad)], None);//Some(FileId(0)));
    match result{
        Err(e)=>{
            println!("Error {}", e.to_live_file_error("", SOURCE));
        }
        Ok((_vertex,pixel))=>{
            //println!("Vertex shader:\n{}\n\nPixel shader:\n{}", vertex,pixel);
            println!("{}", pixel);
        }
    }    
    
    /*
    lr.register_component(id!(main), id!(test), id!(DrawQuad), Box::new(MyShaderFactory {}));
    
    let val = lr.create_component(id!(main), id!(test), &[id!(DrawQuad)]);
    
    match val.unwrap().downcast_mut::<DrawQuad>() {
        Some(comp) => {
            println!("{:?}", comp);
        }
        None => {
            println!("No Value");
        }
    }*/
    
    // ok now we should deserialize MyObj
    // we might wanna plug the shader-compiler in some kind of deserializer as well
}
