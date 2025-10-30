use jni::JNIEnv;

use nfscrs::NFSCRSError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NfscrsJniError{
    #[error("JNIError: {0:?}")]
    JNIError(#[from] jni::errors::Error),
    #[error("NFSCRSError: {0:?}")]
    NFSCRSError(#[from] NFSCRSError),
    #[error("NFSCRSJNIError: {0}")]
    NFSCRSJNIError(String)
}

pub fn throw_nfs_error(env: &mut JNIEnv, err: &NFSCRSError) {
    let (class, msg) = match err {
        NFSCRSError::Connection(e) =>
            ("java/net/ConnectException", format!("Failed to connect: {e}")),
        NFSCRSError::ReadMessage(e) =>
            ("java/io/IOException", format!("Failed to read message: {e}")),
        NFSCRSError::SendMessage(s) =>
            ("java/io/IOException", format!("Failed to send message: {s}")),
        NFSCRSError::Permission(s) =>
            ("java/lang/SecurityException", format!("Permission denied: {s}")),
        NFSCRSError::ReplyDenied(s) =>
            ("java/io/IOException", format!("ONC RPC reply denied: {s}")),
        NFSCRSError::EmptyReplyBody =>
            ("java/io/EOFException", "Empty reply body".to_string()),
        NFSCRSError::NFSStatError(stat) =>
            ("java/io/IOException", format!("NFSStat error: {stat:?}")),
        NFSCRSError::InnerError(e) =>
            ("java/lang/RuntimeException", format!("Inner error: {e}")),
        NFSCRSError::OperationError(s) =>
            ("java/lang/RuntimeException", format!("Operation error: {s}")),
    };

    let _ = env.throw_new(class, msg);
}

pub fn handle_error(env: &mut JNIEnv, e: &NfscrsJniError){
    match e{
        NfscrsJniError::JNIError(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", e.to_string());
        },
        NfscrsJniError::NFSCRSError(e) => {
            throw_nfs_error(env, e);
        }
        NfscrsJniError::NFSCRSJNIError(e) => {
            let _ = env.throw_new("java/lang/RuntimeException", e.to_string());
        }
    }
}