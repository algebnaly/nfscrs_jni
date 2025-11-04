use nfscrs::nfscrs_types::AbsolutePath;
use nfscrs::{NFSClientSession, OpenedFile};

use jni::JNIEnv;
use jni::objects::{JByteArray, JString, JValue};
use jni::{
    objects::JObject,
    sys::{jint, jlong, jobject},
};

use crate::basic_attr_bitmap;
use crate::error::{NfscrsJniError, handle_error, throw_nfs_error};
use crate::file_utils::int_to_open_options;

const NFS4_FILE_READ_RESULT_CLASS_NAME: &str = "com/algebnaly/nfs4c/NFS4FileReadResult";
const NFS4_FILE_WRITE_RESULT_CLASS_NAME: &str = "com/algebnaly/nfs4c/NFS4FileWriteResult";

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

    match read_file(session_ref, opened_file_ref, &byte_buffer, &mut env) {
        Ok(r) => r,
        Err(e) => {
            handle_error(&mut env, &e);
            return std::ptr::null_mut();
        }
    }
}

fn read_file(
    session_ref: &mut NFSClientSession,
    opened_file_ref: &mut OpenedFile,
    byte_buffer: &JObject, // ByteBuffer
    env: &mut JNIEnv,
) -> Result<jobject, NfscrsJniError> {
    let buf_capacity = env
        .call_method(&byte_buffer, "capacity", "()I", &[])
        .and_then(|v| v.i())?;
    let read_result = session_ref.read(opened_file_ref, buf_capacity as usize)?;
    let nfs4_file_read_result_class = env.find_class(NFS4_FILE_READ_RESULT_CLASS_NAME)?;
    let count = read_result.data.len() as jint;
    let result_obj = env.new_object(
        nfs4_file_read_result_class,
        "(ZI)V",
        &[JValue::from(read_result.eof), JValue::from(count)],
    )?;

    let byte_array = env.byte_array_from_slice(&read_result.data)?;

    env.call_method(
        &byte_buffer,
        "put",
        "([B)Ljava/nio/ByteBuffer;",
        &[JValue::Object(&byte_array.into())],
    )?;

    return Ok(result_obj.into_raw());
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_algebnaly_nfs4c_NFS4CNativeBridge_fileWrite(
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

    match write_file(session_ref, opened_file_ref, &byte_buffer, &mut env) {
        Ok(r) => r,
        Err(e) => {
            handle_error(&mut env, &e);
            return std::ptr::null_mut();
        }
    }
}

fn write_file(
    session_ref: &mut NFSClientSession,
    opened_file_ref: &mut OpenedFile,
    byte_buffer: &JObject, // ByteBuffer
    env: &mut JNIEnv,
) -> Result<jobject, NfscrsJniError> {
    let limit = env
        .call_method(byte_buffer, "limit", "()I", &[])
        .and_then(|v| v.i())?;
    
    if limit <= 0 {
        let nfs4_file_write_result_class = env.find_class(NFS4_FILE_WRITE_RESULT_CLASS_NAME)?;
        let result_obj = env.new_object(nfs4_file_write_result_class, "(I)V", &[JValue::from(0)])?;
        return Ok(result_obj.into_raw());
    }
    
    let slice: Vec<u8>;
    let slice_ref: &[u8] = if let Ok(addr) = env.get_direct_buffer_address(byte_buffer.into()) {
        // 直接引用 DirectBuffer 数据
        unsafe { std::slice::from_raw_parts(addr, limit as usize) }
    } else {
        // fallback：普通 Heap ByteBuffer
        slice = {
            let mut v = vec![0u8; limit as usize];
            let byte_array_obj = env
                .call_method(byte_buffer, "array", "()[B", &[])?
                .l()?;
            let v_i8: &mut [i8] = unsafe {
                std::slice::from_raw_parts_mut(v.as_mut_ptr() as *mut i8, v.len())
            };
            let byte_array = JByteArray::from(byte_array_obj);
            env.get_byte_array_region(byte_array, 0, v_i8)?;
            v
        };
        &slice
    };

    let write_result = session_ref.write(opened_file_ref, slice_ref)?;

    let nfs4_file_write_result_class = env.find_class(NFS4_FILE_WRITE_RESULT_CLASS_NAME)?;
    let count = write_result.count;
    let result_obj = env.new_object(
        nfs4_file_write_result_class,
        "(I)V",
        &[JValue::from(count as i32)],
    )?;

    return Ok(result_obj.into_raw());
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_algebnaly_nfs4c_NFS4CNativeBridge_fileClose(
    mut env: JNIEnv,
    _this: JObject,
    session: jlong,
    opened_file: jlong,
) {
    let session_ptr = session as *mut NFSClientSession;
    let session_ref: &mut NFSClientSession = unsafe { &mut *session_ptr };

    let opened_file_ptr = opened_file as *mut OpenedFile;
    let opened_file_ref: &mut OpenedFile = unsafe { &mut *opened_file_ptr };
    match close_file(session_ref, opened_file_ref, &mut env) {
        Ok(_r) => {}
        Err(e) => {
            handle_error(&mut env, &e);
        }
    }
}

fn close_file(
    _session_ref: &mut NFSClientSession,
    _opened_file_ref: &mut OpenedFile,
    _env: &mut JNIEnv,
) -> Result<(), NfscrsJniError> {
    todo!()
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_algebnaly_nfs4c_NFS4CNativeBridge_fileSize(
    mut env: JNIEnv,
    _this: JObject,
    session: jlong,
    opened_file: jlong,
) -> jlong {
    let session_ptr = session as *mut NFSClientSession;
    let session_ref: &mut NFSClientSession = unsafe { &mut *session_ptr };

    let opened_file_ptr = opened_file as *mut OpenedFile;
    let opened_file_ref: &mut OpenedFile = unsafe { &mut *opened_file_ptr };
    match get_file_size_from_opened_file(session_ref, opened_file_ref) {
        Ok(r) => r,
        Err(e) => {
            handle_error(&mut env, &e);
            return 0;
        }
    }
}

fn get_file_size_from_opened_file(
    session_ref: &mut NFSClientSession,
    opened_file_ref: &mut OpenedFile,
) -> Result<i64, NfscrsJniError> {
    let fattr4 = session_ref.get_attr(&opened_file_ref.path, basic_attr_bitmap())?;
    crate::attr_utils::get_file_size(&fattr4).map(|size| size as i64)
}


#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_algebnaly_nfs4c_NFS4CNativeBridge_openFile(
    mut env: JNIEnv,
    _this: JObject,
    session: jlong,
    path: JString,
    open_options: jint,
) -> jlong {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("Invalid path: {e}"));
            return 0;
        }
    };

    let abs_path = match AbsolutePath::try_from(path_str) {
        Ok(p) => p,
        Err(e) => {
            let _ = env.throw_new("java/io/IOException", format!("not absolute path: {e}"));
            return 0;
        }
    };

    let session_ptr = session as *mut NFSClientSession;
    let session_ref: &mut NFSClientSession = unsafe { &mut *session_ptr };

    let opts = int_to_open_options(open_options);

    let opened_file = match session_ref.open_file_and_comfirm(&abs_path, opts) {
        Ok(r) => r,
        Err(e) => {
            throw_nfs_error(&mut env, &e);
            return 0;
        }
    };
    let file = Box::new(opened_file);
    let file_ptr = Box::into_raw(file);
    file_ptr as jlong
}
