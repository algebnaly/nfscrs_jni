use jni::JNIEnv;
use nfscrs::{
    fattr4::{fattr4_names, FAttr4, FAttr4Type}, nfs4_types::{NFSFType4, NFSTime4}
};

pub fn get_access_time(fattr4: &FAttr4, env: &mut JNIEnv) -> NFSTime4 {
    if let Ok(fattr4type) = fattr4.fetch_attr(fattr4_names::FATTR4_TIME_ACCESS)
        && let FAttr4Type::FATTR4_TIME_ACCESS(t) = fattr4type
    {
        t
    } else {
        let _ = env.throw_new("java/io/IOException", format!("nfs4 protocol error"));
        unreachable!()
    }
}

pub fn get_modify_time(fattr4: &FAttr4, env: &mut JNIEnv) -> NFSTime4 {
    if let Ok(fattr4type) = fattr4.fetch_attr(fattr4_names::FATTR4_TIME_MODIFY)
        && let FAttr4Type::FATTR4_TIME_MODIFY(t) = fattr4type
    {
        t
    } else {
        let _ = env.throw_new("java/io/IOException", format!("nfs4 protocol error"));
        unreachable!()
    }
}

pub fn get_create_time(fattr4: &FAttr4, env: &mut JNIEnv) -> NFSTime4 {
    if let Ok(fattr4type) = fattr4.fetch_attr(fattr4_names::FATTR4_TIME_CREATE)
        && let FAttr4Type::FATTR4_TIME_CREATE(t) = fattr4type
    {
        t
    } else {
        let _ = env.throw_new("java/io/IOException", format!("nfs4 protocol error"));
        unreachable!()
    }
}

pub fn get_filetype(fattr4: &FAttr4, env: &mut JNIEnv) -> NFSFType4 {
    if let Ok(fattr4type) = fattr4.fetch_attr(fattr4_names::FATTR4_TYPE)
        && let FAttr4Type::FATTR4_TYPE(t) = fattr4type
    {
        t
    } else {
        let _ = env.throw_new("java/io/IOException", format!("nfs4 protocol error"));
        unreachable!()
    }
}

pub fn get_file_size(fattr4: &FAttr4, env: &mut JNIEnv) -> u64 {
    if let Ok(fattr4type) = fattr4.fetch_attr(fattr4_names::FATTR4_SIZE)
        && let FAttr4Type::FATTR4_SIZE(t) = fattr4type
    {
        t
    } else {
        let _ = env.throw_new("java/io/IOException", format!("nfs4 protocol error"));
        unreachable!()
    }
}

pub fn get_file_mode(fattr4: &FAttr4, env: &mut JNIEnv) -> u32 {
    if let Ok(fattr4type) = fattr4.fetch_attr(fattr4_names::FATTR4_MODE)
        && let FAttr4Type::FATTR4_MODE(t) = fattr4type
    {
        t
    } else {
        let _ = env.throw_new("java/io/IOException", format!("nfs4 protocol error"));
        unreachable!()
    }
}
