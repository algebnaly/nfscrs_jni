use nfscrs::fattr4::set_bitmap;
use nfscrs::nfs4_types::{BitMap4, NFSFType4};
use nfscrs::nfs4_utils::nfs4time_to_miliseconds;
use nfscrs::nfscrs_types::AbsolutePath;
use nfscrs::{self, NFSClientBuilder, NFSClientSession};

use jni::objects::{JClass, JStaticMethodID, JValue};
use jni::JNIEnv;
use jni::{
    objects::{JObject, JString},
    sys::{jint, jlong, jobject},
};

use crate::attr_utils::{
    get_access_time, get_create_time, get_file_mode, get_file_size, get_filetype, get_modify_time,
};
use crate::error::{handle_error, throw_nfs_error};

mod attr_utils;
mod error;
mod file_utils;
mod file_ops;
mod jni_utils;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_algebnaly_nfs4c_NFS4CNativeBridge_getClientSession(
    mut env: JNIEnv,
    _this: JObject,
    uid: jint,
    gid: jint,
    remote_addr: JString,
) -> jlong {
    let r_addr_result = match env.get_string(&remote_addr) {
        Ok(s) => s,
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("Invalid remote_addr: {e}"));
            return 0;
        }
    };

    let r_addr = match r_addr_result.to_str() {
        Ok(s) => s,
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("UTF-8 decode error: {e}"));
            return 0;
        }
    };

    let parsed_addr = match r_addr.parse() {
        Ok(addr) => addr,
        Err(e) => {
            let _ = env.throw_new(
                "java/net/URISyntaxException",
                format!("Invalid address: {e}"),
            );
            return 0;
        }
    };

    let session =
        match NFSClientBuilder::new(uid as u32, gid as u32, parsed_addr).establish_session() {
            Ok(session) => session,
            Err(e) => {
                throw_nfs_error(&mut env, &e);
                return 0;
            }
        };

    let addr = Box::into_raw(Box::new(session));
    addr as jlong
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_algebnaly_nfs4c_NFS4CNativeBridge_listDir(
    mut env: JNIEnv,
    _this: JObject,
    session: jlong,
    path: JString,
) -> jobject {
    let path_result = match env.get_string(&path) {
        Ok(p) => p,
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("Invalid path: {e}"));
            return std::ptr::null_mut();
        }
    };
    let path: &str = match path_result.to_str() {
        Ok(p) => p,
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("utf8 decode error: {e}"));
            return std::ptr::null_mut();
        }
    };

    let abs_path = match AbsolutePath::try_from(path) {
        Ok(p) => p,
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("not absolute path: {e}"));
            return std::ptr::null_mut();
        }
    };

    let session_ptr = session as *mut NFSClientSession;
    let session_ref: &mut NFSClientSession = unsafe { &mut *session_ptr };
    let r = match session_ref.list_dir(&abs_path) {
        Ok(r) => r,
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("list dir error: {e}"));
            return std::ptr::null_mut();
        }
    };

    let rr = r.iter().map(|e| String::from_utf8_lossy(&e.name));
    let array_list_class = match env.find_class("java/util/ArrayList") {
        Ok(class) => class,
        Err(e) => {
            let _ = env.throw_new(
                "java/io/IOException",
                format!("ArrayList class not found: {e}"),
            );
            return std::ptr::null_mut();
        }
    };
    let array_list_obj = match env.new_object(&array_list_class, "()V", &[]) {
        Ok(obj) => obj,
        Err(e) => {
            let _ = env.throw_new(
                "java/io/IOException",
                format!("Failed to create ArrayList: {e}"),
            );
            return std::ptr::null_mut();
        }
    };
    let add_method = match env.get_method_id(&array_list_class, "add", "(Ljava/lang/Object;)Z") {
        Ok(method) => method,
        Err(e) => {
            let _ = env.throw_new(
                "java/io/IOException",
                format!("Failed to get add() method: {e}"),
            );
            return std::ptr::null_mut();
        }
    };

    for name in rr {
        let jname: JString = match env.new_string(name) {
            Ok(jstring) => jstring,
            Err(e) => {
                let _ = env.throw_new(
                    "java/io/IOException",
                    format!("Failed to create JString: {e}"),
                );
                return std::ptr::null_mut();
            }
        };
        let jval = JValue::Object(&jname).as_jni();
        unsafe {
            match env.call_method_unchecked(
                &array_list_obj,
                add_method,
                jni::signature::ReturnType::Primitive(jni::signature::Primitive::Boolean),
                &[jval],
            ) {
                Ok(_) => {}
                Err(e) => {
                    let _ = env.throw_new(
                        "java/io/IOException",
                        format!("Failed to call add() method: {e}"),
                    );
                    return std::ptr::null_mut();
                }
            };
        }
    }
    array_list_obj.into_raw()
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_algebnaly_nfs4c_NFS4CNativeBridge_readAttr(
    mut env: JNIEnv,
    _this: JObject,
    session: jlong,
    path: JString,
) -> jobject {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("Invalid path: {e}"));
            return std::ptr::null_mut();
        }
    };

    let abs_path = match AbsolutePath::try_from(path_str) {
        Ok(p) => p,
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("not absolute path: {e}"));
            return std::ptr::null_mut();
        }
    };

    let session_ptr = session as *mut NFSClientSession;
    let session_ref: &mut NFSClientSession = unsafe { &mut *session_ptr };

    let fattr4 = match session_ref.get_attr(&abs_path, basic_attr_bitmap()) {
        Ok(s) => s,
        Err(e) => {
            crate::error::throw_nfs_error(&mut env, &e);
            return std::ptr::null_mut();
        }
    };

    let filetype = get_filetype(&fattr4, &mut env);

    let filesize = match get_file_size(&fattr4){
        Ok(size) => size,
        Err(e) => {
            handle_error(&mut env, &e);
            return std::ptr::null_mut();
        }
    };

    let filemode = get_file_mode(&fattr4, &mut env) as i32;
    let access_time = get_access_time(&fattr4, &mut env);
    let access_time_millis: jlong = nfs4time_to_miliseconds(&access_time);

    let modify_time = get_modify_time(&fattr4, &mut env);
    let modify_time_millis: jlong = nfs4time_to_miliseconds(&modify_time);

    let create_time = get_create_time(&fattr4, &mut env);
    let create_time_millis: jlong = nfs4time_to_miliseconds(&create_time);

    let filetime_class = env
        .find_class("java/nio/file/attribute/FileTime")
        .expect("FileTime class not found");
    let from_millis = env
        .get_static_method_id(
            &filetime_class,
            "fromMillis",
            "(J)Ljava/nio/file/attribute/FileTime;",
        )
        .expect("FileTime.fromMillis not found");

    let last_access_time =
        match create_filetime(&filetime_class, from_millis, access_time_millis, &mut env) {
            Ok(time) => time,
            Err(e) => {
                let _ = env.throw_new(
                    "java/io/IOException",
                    format!("Failed to create FileTime: {}", e),
                );
                return std::ptr::null_mut();
            }
        };
    let last_modify_time =
        match create_filetime(&filetime_class, from_millis, modify_time_millis, &mut env) {
            Ok(time) => time,
            Err(e) => {
                let _ = env.throw_new(
                    "java/io/IOException",
                    format!("Failed to create FileTime: {}", e),
                );
                return std::ptr::null_mut();
            }
        };

    let creation_time =
        match create_filetime(&filetime_class, from_millis, create_time_millis, &mut env) {
            Ok(time) => time,
            Err(e) => {
                let _ = env.throw_new(
                    "java/io/IOException",
                    format!("Failed to create FileTime: {}", e),
                );
                return std::ptr::null_mut();
            }
        };

    let nfs_attrs_class = match env.find_class("com/algebnaly/nfs4c/NFS4FileAttributes") {
        Ok(class) => class,
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("class not found: {e}"));
            return std::ptr::null_mut();
        }
    };

    let ctor_sig = "(Ljava/nio/file/attribute/FileTime;Ljava/nio/file/attribute/FileTime;Ljava/nio/file/attribute/FileTime;ZZZZJILjava/lang/Object;)V";

    let is_regular = matches!(filetype, NFSFType4::NF4REG);
    let is_directory = matches!(filetype, NFSFType4::NF4DIR);
    let is_symlink = matches!(filetype, NFSFType4::NF4LNK);
    let is_other = !is_regular && !is_directory && !is_symlink;

    let obj = env
        .new_object(
            nfs_attrs_class,
            ctor_sig,
            &[
                JValue::Object(&last_access_time),
                JValue::Object(&last_modify_time),
                JValue::Object(&creation_time),
                JValue::Bool(if is_regular { 1 } else { 0 }),
                JValue::Bool(if is_directory { 1 } else { 0 }),
                JValue::Bool(if is_symlink { 1 } else { 0 }),
                JValue::Bool(if is_other { 1 } else { 0 }),
                JValue::Long(filesize as jlong),
                JValue::Int(filemode),
                JValue::Object(&JObject::null()),
            ],
        )
        .expect("Failed to create NFS4FileAttributes");

    obj.into_raw()
}




fn basic_attr_bitmap() -> BitMap4 {
    use nfscrs::fattr4::fattr4_names;
    let mut bitmap = BitMap4::new();
    set_bitmap(&mut bitmap, fattr4_names::FATTR4_TIME_ACCESS);
    set_bitmap(&mut bitmap, fattr4_names::FATTR4_TIME_MODIFY);
    set_bitmap(&mut bitmap, fattr4_names::FATTR4_TIME_CREATE);
    set_bitmap(&mut bitmap, fattr4_names::FATTR4_TYPE);
    set_bitmap(&mut bitmap, fattr4_names::FATTR4_SIZE);
    set_bitmap(&mut bitmap, fattr4_names::FATTR4_MODE);
    bitmap
}

fn create_filetime<'a>(
    filetime_class: &JClass<'a>,
    from_millis: JStaticMethodID,
    time_millis: i64,
    env: &mut JNIEnv<'a>,
) -> Result<JObject<'a>, std::io::Error> {
    unsafe {
        env.call_static_method_unchecked(
            filetime_class,
            from_millis,
            jni::signature::ReturnType::Object,
            &[JValue::Long(time_millis).as_jni()],
        )
        .and_then(|v| v.l())
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create FileTime: {}", e),
            )
        })
    }
}
