use nfscrs::{NFSClientSession, OpenedFile};

use jni::JNIEnv;
use jni::objects::JValue;
use jni::{
    objects::JObject,
    sys::{jint, jlong, jobject},
};

use crate::error::{NfscrsJniError, throw_nfs_error};

const NFS4_FILE_READ_RESULT_NAME: &str = "com/algebnaly/nfs4c/NFS4FileReadResult";

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_algebnaly_nfs4c_NFS4CNativeBridge_fileRead(
    mut env: JNIEnv,
    _this: JObject,
    session: jlong,
    opened_file: jlong,
    byte_buffer: JObject,
) -> jobject {
    let session_ptr = session as *mut NFSClientSession;
    let session_ref: &mut NFSClientSession = unsafe { &mut *session_ptr };
    
    let opened_file_ptr = opened_file as *mut OpenedFile;
    let opened_file_ref: &mut OpenedFile = unsafe { &mut *opened_file_ptr };
    
    match read_file(session_ref, opened_file_ref, &byte_buffer, &mut env){
        Ok(r) => {
            r
        },
        Err(e) => {
            match e {
                NfscrsJniError::JNIError(e) => {
                    let _ = env.throw_new("java/lang/RuntimeException", e.to_string());
                    return std::ptr::null_mut();
                }
                NfscrsJniError::NFSCRSError(e) => {
                    throw_nfs_error(&mut env, e);
                    return std::ptr::null_mut();
                }
            }
        }
    }
}

fn read_file(
    session_ref: &mut NFSClientSession,
    opened_file_ref: &mut OpenedFile,
    byte_buffer: &JObject, // ByteBuffer
    env: &mut JNIEnv
)-> Result<jobject, NfscrsJniError>{
    let buf_capacity = env
        .call_method(&byte_buffer, "capacity", "()I", &[])
        .and_then(|v| v.i())?;
    
    let read_result = session_ref.read(opened_file_ref, buf_capacity as usize)?;
    let nfs4_file_read_result_class = env.find_class(NFS4_FILE_READ_RESULT_NAME)?;
    let count = read_result.data.len() as jint;
    let result_obj = env.new_object(nfs4_file_read_result_class, "(ZI)V", &[JValue::from(read_result.eof), JValue::from(count)])?;
    
    let byte_array = env.byte_array_from_slice(&read_result.data)?;
    
    env.call_method(&byte_buffer, "put", "([B)Ljava/nio/ByteBuffer;", &[JValue::Object(&byte_array.into())])?;
    
    return Ok(result_obj.into_raw())
}
