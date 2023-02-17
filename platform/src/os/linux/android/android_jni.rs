#![allow(dead_code)]

use {
    self::super::{
        jni_sys::{jclass, jsize, jint, jlong, jstring, jobject, JNIEnv, JNI_ABORT},
    },
    crate::{
        area::Area,
        makepad_math::*,
        event::*,
        cx::{Cx,AndroidParams},
    },
    std::{
        cell::Cell,
        ffi::{CString},
        marker::PhantomData,
    },
};

/// This struct corresponds to the `Makepad.Callback` interface in Java (which is implemented by
/// the `MakepadSurface` class) and enables us to call methods on that interface while hiding as
/// much of the Java native interface from our Rust code as possible.
///
/// The lifetime is necessary here because object pointers in Java are not stable, so the object
/// pointer wrapped by this struct is really only valid for the duration of each native call.
pub struct AndroidToJava<'a> {
    env: *mut JNIEnv,
    callback: jobject,
    phantom: PhantomData<&'a ()>,
}

impl<'a> AndroidToJava<'a> {
    pub fn get_env(&self)->*mut JNIEnv{
        self.env
    }
    
    /// Swaps the buffers of the MakepadSurface.
    pub fn swap_buffers(&self) {
        unsafe {
            let class = ((**self.env).GetObjectClass.unwrap())(self.env, self.callback);
            let name = CString::new("swapBuffers").unwrap();
            let signature = CString::new("()V").unwrap();
            let method_id = ((**self.env).GetMethodID.unwrap())(
                self.env,
                class,
                name.as_ptr(),
                signature.as_ptr(),
            );
            ((**self.env).CallVoidMethod.unwrap())(self.env, self.callback, method_id);
        }
    }
    
    /// Schedules a call to `Cx::draw`.
    ///
    /// This works by marking the MakepadSurface as dirty and therefore *should* synchronize
    /// correctly with vsync.
    pub fn schedule_redraw(&self) {
        unsafe {
            let class = ((**self.env).GetObjectClass.unwrap())(self.env, self.callback);
            let name = CString::new("scheduleRedraw").unwrap();
            let signature = CString::new("()V").unwrap();
            let method_id = ((**self.env).GetMethodID.unwrap())(
                self.env,
                class,
                name.as_ptr(),
                signature.as_ptr(),
            );
            ((**self.env).CallVoidMethod.unwrap())(self.env, self.callback, method_id);
        }
    }
    
    /// Schedules a timeout with the given `id` and `delay`, where `delay` is given in
    /// milliseconds.
    ///
    /// It is your responsibility to make sure that timeout ids are unique.
    pub fn schedule_timeout(&self, id: i64, delay: i64) {
        unsafe {
            let class = ((**self.env).GetObjectClass.unwrap())(self.env, self.callback);
            let name = CString::new("scheduleTimeout").unwrap();
            let signature = CString::new("(JJ)V").unwrap();
            let method_id = ((**self.env).GetMethodID.unwrap())(
                self.env,
                class,
                name.as_ptr(),
                signature.as_ptr(),
            );
            ((**self.env).CallVoidMethod.unwrap())(self.env, self.callback, method_id, id, delay);
        }
    }
    
    /// Cancels the timeout with the given id.
    ///
    /// It is your responsibility to make sure that timeout ids are unique.
    pub fn cancel_timeout(&self, id: i64) {
        unsafe {
            let class = ((**self.env).GetObjectClass.unwrap())(self.env, self.callback);
            let name = CString::new("cancelTimeout").unwrap();
            let signature = CString::new("(J)V").unwrap();
            let method_id = ((**self.env).GetMethodID.unwrap())(
                self.env,
                class,
                name.as_ptr(),
                signature.as_ptr(),
            );
            ((**self.env).CallVoidMethod.unwrap())(self.env, self.callback, method_id, id);
        }
    }
    
        
    /// reads an asset
    ///
    ///
    pub fn read_asset(&self, file:&str)->Option<Vec<u8>> {
        unsafe {
            let class = ((**self.env).GetObjectClass.unwrap())(self.env, self.callback);
            
            let name = CString::new("readAsset").unwrap();
            let signature = CString::new("(Ljava/lang/String;)[B").unwrap();
            let file = CString::new(file).unwrap();
            let file = ((**self.env).NewStringUTF.unwrap())(self.env, file.as_ptr());
            let method_id = ((**self.env).GetMethodID.unwrap())(
                self.env,
                class,
                name.as_ptr(),
                signature.as_ptr(),
            );
            let byte_array = ((**self.env).CallObjectMethod.unwrap())(self.env, self.callback, method_id, file);
            if byte_array == std::ptr::null_mut(){
                return None
            }
            else{
                return Some(java_byte_array_to_vec(self.env, byte_array));
            }
        }
    }
    
    pub fn get_audio_devices(&self, flag:jlong)->Vec<String> {
        unsafe {
            let class = ((**self.env).GetObjectClass.unwrap())(self.env, self.callback);
            
            let name = CString::new("getAudioDevices").unwrap();
            let signature = CString::new("(J)[Ljava/lang/String;").unwrap();
            let method_id = ((**self.env).GetMethodID.unwrap())(
                self.env,  
                class,
                name.as_ptr(), 
                signature.as_ptr(), 
            );
            let string_array = ((**self.env).CallObjectMethod.unwrap())(self.env, self.callback, method_id, flag);
            return java_string_array_to_vec(self.env, string_array);
        }
    }
     
    pub fn open_all_midi_devices(&self){
        unsafe {
            let class = ((**self.env).GetObjectClass.unwrap())(self.env, self.callback);
            let name = CString::new("openAllMidiDevices").unwrap();
            let signature = CString::new("()V").unwrap();
            let method_id = ((**self.env).GetMethodID.unwrap())(
                self.env,  
                class,
                name.as_ptr(),
                signature.as_ptr(), 
            );
            ((**self.env).CallVoidMethod.unwrap())(self.env, self.callback, method_id);
        }
    }
}

// The functions here correspond to the static functions on the `Makepad` class in Java.

    
#[no_mangle]
pub unsafe extern "C" fn Java_nl_makepad_android_Makepad_dropCx(_: JNIEnv, _: jclass, _app: jlong) {
    //log!("DROP!"); 
    //drop(Box::from_raw(app as *mut Cx));
}

unsafe fn jstring_to_string(env:*mut JNIEnv, java_string: jstring)->String{
    let chars = (**env).GetStringUTFChars.unwrap()(env, java_string, std::ptr::null_mut());
    let rust_string = std::ffi::CStr::from_ptr(chars).to_str().unwrap().to_string();
    (**env).ReleaseStringUTFChars.unwrap()(env, java_string, chars);
    rust_string
}

unsafe fn java_string_array_to_vec(env:*mut JNIEnv, object_array: jobject)->Vec<String>{
    if object_array == std::ptr::null_mut(){
        return Vec::new();
    }
    let mut out = Vec::new();
    let length = (**env).GetArrayLength.unwrap()(env, object_array);
    for i in 0..length{
        let string = (**env).GetObjectArrayElement.unwrap()(env, object_array, i as jsize);
        out.push(jstring_to_string(env, string));
    }
    out
}

unsafe fn java_byte_array_to_vec(env:*mut JNIEnv, byte_array: jobject)->Vec<u8>{
    let bytes = (**env).GetByteArrayElements.unwrap()(env, byte_array, std::ptr::null_mut());
    let length = (**env).GetArrayLength.unwrap()(env, byte_array);
    let mut out_bytes = Vec::new();
    let slice = std::slice::from_raw_parts(bytes as *const u8, length as usize);
    out_bytes.extend_from_slice(slice);
    (**env).ReleaseByteArrayElements.unwrap()(env, byte_array, bytes, JNI_ABORT);
    out_bytes
}


pub struct AndroidInitParams{
    pub cache_path: String,
}

#[no_mangle]
pub unsafe extern "C" fn Java_nl_makepad_android_Makepad_init(
    env: *mut JNIEnv,
    _: jclass,
    cx: jlong,
    cache_path: jstring,
    callback: jobject,
) {
    (*(cx as *mut Cx)).from_java_init(
        AndroidParams{ 
            cache_path: jstring_to_string(env, cache_path),
        },
        AndroidToJava { 
        env,
        callback,
        phantom: PhantomData,
    });
}

#[no_mangle] 
pub unsafe extern "C" fn Java_nl_makepad_android_Makepad_resize(
    env: *mut JNIEnv,
    _: jclass,
    cx: jlong,
    width: jint,
    height: jint,
    callback: jobject,
) {
    (*(cx as *mut Cx)).from_java_resize(
        width,
        height,
        AndroidToJava {
            env,
            callback,
            phantom: PhantomData,
        },
    );
}

#[no_mangle]
pub unsafe extern "C" fn Java_nl_makepad_android_Makepad_draw(
    env: *mut JNIEnv,
    _: jclass,
    cx: jlong,
    callback: jobject,
) {
    (*(cx as *mut Cx)).from_java_draw(AndroidToJava {
        env,
        callback,
        phantom: PhantomData,
    });
}

#[no_mangle]
pub unsafe extern "C" fn Java_nl_makepad_android_Makepad_touch(
    env: *mut JNIEnv,
    _: jclass,
    cx: jlong,
    event: jobject,
    callback: jobject,
) {
    let action_masked = unsafe {
        let class = ((**env).GetObjectClass.unwrap())(env, event);
        let name = CString::new("getActionMasked").unwrap();
        let signature = CString::new("()I").unwrap();
        let method_id =
        ((**env).GetMethodID.unwrap())(env, class, name.as_ptr(), signature.as_ptr());
        ((**env).CallIntMethod.unwrap())(env, event, method_id)
    };
    let action_index = unsafe {
        let class = ((**env).GetObjectClass.unwrap())(env, event);
        let name = CString::new("getActionIndex").unwrap();
        let signature = CString::new("()I").unwrap();
        let method_id =
        ((**env).GetMethodID.unwrap())(env, class, name.as_ptr(), signature.as_ptr());
        ((**env).CallIntMethod.unwrap())(env, event, method_id)
    };
    
    let touch_count = unsafe {
        let class = ((**env).GetObjectClass.unwrap())(env, event);
        let name = CString::new("getPointerCount").unwrap();
        let signature = CString::new("()I").unwrap();
        let method_id =
        ((**env).GetMethodID.unwrap())(env, class, name.as_ptr(), signature.as_ptr());
        ((**env).CallIntMethod.unwrap())(env, event, method_id)
    };
    
    let mut touches = Vec::with_capacity(touch_count as usize);
    for touch_index in 0..touch_count {
        let id = unsafe {
            let class = ((**env).GetObjectClass.unwrap())(env, event);
            let name = CString::new("getPointerId").unwrap();
            let signature = CString::new("(I)I").unwrap();
            let method_id =
            ((**env).GetMethodID.unwrap())(env, class, name.as_ptr(), signature.as_ptr());
            ((**env).CallIntMethod.unwrap())(env, event, method_id, touch_index)
        };
        
        let x = unsafe {
            let class = ((**env).GetObjectClass.unwrap())(env, event);
            let name = CString::new("getX").unwrap();
            let signature = CString::new("(I)F").unwrap();
            let method_id =
            ((**env).GetMethodID.unwrap())(env, class, name.as_ptr(), signature.as_ptr());
            ((**env).CallFloatMethod.unwrap())(env, event, method_id, touch_index)
        };
        
        let y = unsafe {
            let class = ((**env).GetObjectClass.unwrap())(env, event);
            let name = CString::new("getY").unwrap();
            let signature = CString::new("(I)F").unwrap();
            let method_id =
            ((**env).GetMethodID.unwrap())(env, class, name.as_ptr(), signature.as_ptr());
            ((**env).CallFloatMethod.unwrap())(env, event, method_id, touch_index)
        };
        
        let rotation_angle = unsafe {
            let class = ((**env).GetObjectClass.unwrap())(env, event);
            let name = CString::new("getOrientation").unwrap();
            let signature = CString::new("(I)F").unwrap();
            let method_id =
            ((**env).GetMethodID.unwrap())(env, class, name.as_ptr(), signature.as_ptr());
            ((**env).CallFloatMethod.unwrap())(env, event, method_id, touch_index)
        } as f64;
        
        let force = unsafe {
            let class = ((**env).GetObjectClass.unwrap())(env, event);
            let name = CString::new("getPressure").unwrap();
            let signature = CString::new("(I)F").unwrap();
            let method_id =
            ((**env).GetMethodID.unwrap())(env, class, name.as_ptr(), signature.as_ptr());
            ((**env).CallFloatMethod.unwrap())(env, event, method_id, touch_index)
        } as f64;
        
        touches.push(TouchPoint {
            state: {
                if action_index == touch_index {
                    match action_masked {
                        0 | 5 => TouchState::Start,
                        1 | 6 => TouchState::Stop,
                        2 => TouchState::Move,
                        _ => return,
                    }
                }
                else {
                    TouchState::Move
                }
            },
            uid: id as u64,
            rotation_angle,
            force,
            radius: dvec2(1.0,1.0),
            handled: Cell::new(Area::Empty),
            sweep_lock: Cell::new(Area::Empty),
            abs: dvec2(x as f64, y as f64),
        });
    }
    
    (*(cx as *mut Cx)).from_java_touch_update(
        touches,
        AndroidToJava {
            env,
            callback,
            phantom: PhantomData,
        },
    );
}

#[no_mangle]
pub unsafe extern "C" fn Java_nl_makepad_android_Makepad_timeout(
    env: *mut JNIEnv,
    _: jclass,
    cx: jlong,
    id: jlong,
    callback: jobject,
) {
    (*(cx as *mut Cx)).from_java_timeout(
        id,
        AndroidToJava {
            env,
            callback,
            phantom: PhantomData,
        },
    );
}

#[no_mangle]
pub unsafe extern "C" fn Java_nl_makepad_android_Makepad_midiDevice(
    env: *mut JNIEnv,
    _: jclass,
    cx: jlong,
    name: jstring,
    midi_device: jobject,
    callback: jobject,
) {
    (*(cx as *mut Cx)).from_java_midi_device(
        jstring_to_string(env, name),
        midi_device,
        AndroidToJava {
            env,
            callback,
            phantom: PhantomData,
        },
    );
}