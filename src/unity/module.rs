use std::{ffi::{c_void, CString}, marker::PhantomData, mem::size_of, ops::Deref, path::PathBuf};
use thiserror::Error;
use winapi::{shared::minwindef::FALSE, um::{libloaderapi::LoadLibraryA, processthreadsapi::GetCurrentProcess, psapi::{GetModuleInformation, MODULEINFO}}};

#[derive(Debug, Error)]
pub enum ModuleError {
  #[error("failed to convert PathBuf to str")]
  PathBufToStr,
  #[error("failed to create a CString object")]
  CreateCString,
  #[error("failed to load a library")]
  LoadLibrary,
  #[error("failed to get module information")]
  ModuleInformation
}

pub struct Module {
  pub handle: *mut c_void,
  pub size: usize
}

impl Module {
  pub fn load(path: PathBuf) -> Result<Self, ModuleError> {
    unsafe {
      let path_str = path.to_str().ok_or_else(|| ModuleError::PathBufToStr)?;
      let native = CString::new(path_str).map_err(|_| ModuleError::CreateCString)?;
      let handle = LoadLibraryA(native.as_ptr());
  
      if handle.is_null() {
        return Err(ModuleError::LoadLibrary);
      }

      let mut module_info: MODULEINFO = std::mem::zeroed();
      let res = GetModuleInformation(GetCurrentProcess(), handle.cast(), &mut module_info, size_of::<MODULEINFO>() as u32);

      if res == FALSE {
        return Err(ModuleError::ModuleInformation);
      }

      let size = module_info.SizeOfImage as usize;

      Ok(Module {
        handle: handle.cast(),
        size: size
      })
    }
  }
}

pub struct MethodPtr<T> {
  pub ptr: *mut c_void,
  pd: PhantomData<T>
}

impl<T> Deref for MethodPtr<T> {
  type Target = T;

  fn deref(&self) -> &T {
    unsafe { &*(&self.ptr as *const *mut _ as *const T) }
  }
}

impl<T> Clone for MethodPtr<T> {
  fn clone(&self) -> Self {
    MethodPtr { ..*self }
  }
}

pub fn get_method_ptr<T>(offset: usize) -> Option<MethodPtr<T>> {
  unsafe {
    let ptr = *(offset as *mut usize);

    if ptr == 0 {
      return None;
    }

    Some(MethodPtr {
      ptr: ptr as *mut c_void,
      pd: PhantomData
    })
  }
}