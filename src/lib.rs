use jni::objects::JValue;
use nfscrs::nfscrs_types::AbsolutePath;
use nfscrs::{self, NFSClientBuilder, NFSClientSession};

use jni::JNIEnv;
use jni::{
    objects::{JObject, JString},
    sys::{jint, jlong, jobject},
};

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_algebnaly_nfs4c_NFS4CNativeBridge_getClientSession(
    mut env: JNIEnv,
    _this: JObject,
    uid: jint,
    gid: jint,
    remote_addr: JString,
) -> jlong {
    let r_addr_result = env.get_string(&remote_addr).unwrap();
    let r_addr: &str = r_addr_result.to_str().unwrap();
    let session = NFSClientBuilder::new(uid as u32, gid as u32, r_addr.parse().unwrap())
        .establish_session()
        .unwrap();
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
    let path_result = env.get_string(&path).unwrap();
    let path: &str = path_result.to_str().unwrap();
    let session_ptr = session as *mut NFSClientSession;
    let session_handler: &mut NFSClientSession = unsafe { &mut *session_ptr };
    let r = session_handler
        .list_dir(&AbsolutePath::try_from(path).unwrap())
        .unwrap();
    let rr = r.iter().map(|e| String::from_utf8_lossy(&e.name));
    let array_list_class = env
        .find_class("java/util/ArrayList")
        .expect("ArrayList class not found");
    let array_list_obj = env
        .new_object(&array_list_class, "()V", &[])
        .expect("Failed to create ArrayList");
    let add_method = env
        .get_method_id(&array_list_class, "add", "(Ljava/lang/Object;)Z")
        .expect("add() not found");
    for name in rr {
        let jname: JString = env.new_string(name).unwrap();
        let jval = JValue::Object(&jname).as_jni();
        unsafe {
            env.call_method_unchecked(
                &array_list_obj,
                add_method,
                jni::signature::ReturnType::Primitive(jni::signature::Primitive::Boolean),
                &[jval],
            )
            .unwrap();
        }
    }
    array_list_obj.into_raw()
}
