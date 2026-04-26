//! Vulkan surface creation against `raw-window-handle` 0.6 (the version Bevy
//! exposes through `bevy_window::RawHandleWrapper`). Replaces the 0.5-based
//! helpers in `examples/window/utils` for the Bevy integration.

use std::ffi::c_char;
use std::rc::Rc;

use ash::{ext, khr, vk};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use sunray::{
    error::{SrError, SrResult},
    vulkan_abstraction,
};

/// Instance extensions required to later create a surface from `display_handle`.
/// Returned slice is `'static` so it can be passed directly to
/// `sunray::Renderer::new_with_surface`.
pub fn enumerate_required_extensions(
    display_handle: RawDisplayHandle,
) -> SrResult<&'static [*const c_char]> {
    let exts: &'static [*const c_char] = match display_handle {
        RawDisplayHandle::Windows(_) => {
            const E: [*const c_char; 2] =
                [khr::surface::NAME.as_ptr(), khr::win32_surface::NAME.as_ptr()];
            &E
        }
        RawDisplayHandle::Wayland(_) => {
            const E: [*const c_char; 2] =
                [khr::surface::NAME.as_ptr(), khr::wayland_surface::NAME.as_ptr()];
            &E
        }
        RawDisplayHandle::Xlib(_) => {
            const E: [*const c_char; 2] =
                [khr::surface::NAME.as_ptr(), khr::xlib_surface::NAME.as_ptr()];
            &E
        }
        RawDisplayHandle::Xcb(_) => {
            const E: [*const c_char; 2] =
                [khr::surface::NAME.as_ptr(), khr::xcb_surface::NAME.as_ptr()];
            &E
        }
        RawDisplayHandle::Android(_) => {
            const E: [*const c_char; 2] =
                [khr::surface::NAME.as_ptr(), khr::android_surface::NAME.as_ptr()];
            &E
        }
        RawDisplayHandle::AppKit(_) | RawDisplayHandle::UiKit(_) => {
            const E: [*const c_char; 2] =
                [khr::surface::NAME.as_ptr(), ext::metal_surface::NAME.as_ptr()];
            &E
        }
        _ => return Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT.into()),
    };
    Ok(exts)
}

pub fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    display_handle: RawDisplayHandle,
    window_handle: RawWindowHandle,
) -> SrResult<vk::SurfaceKHR> {
    match (display_handle, window_handle) {
        (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(w)) => {
            let info = vk::Win32SurfaceCreateInfoKHR::default()
                .hwnd(w.hwnd.get())
                .hinstance(w.hinstance.map(|h| h.get()).unwrap_or(0));
            let f = khr::win32_surface::Instance::new(entry, instance);
            unsafe { f.create_win32_surface(&info, None) }.map_err(SrError::from)
        }
        (RawDisplayHandle::Wayland(d), RawWindowHandle::Wayland(w)) => {
            let info = vk::WaylandSurfaceCreateInfoKHR::default()
                .display(d.display.as_ptr())
                .surface(w.surface.as_ptr());
            let f = khr::wayland_surface::Instance::new(entry, instance);
            unsafe { f.create_wayland_surface(&info, None) }.map_err(SrError::from)
        }
        (RawDisplayHandle::Xlib(d), RawWindowHandle::Xlib(w)) => {
            let dpy = d
                .display
                .map(|p| p.as_ptr())
                .unwrap_or(std::ptr::null_mut()) as *mut _;
            let info = vk::XlibSurfaceCreateInfoKHR::default()
                .dpy(dpy)
                .window(w.window);
            let f = khr::xlib_surface::Instance::new(entry, instance);
            unsafe { f.create_xlib_surface(&info, None) }.map_err(SrError::from)
        }
        (RawDisplayHandle::Xcb(d), RawWindowHandle::Xcb(w)) => {
            let conn = d
                .connection
                .map(|p| p.as_ptr())
                .unwrap_or(std::ptr::null_mut());
            let info = vk::XcbSurfaceCreateInfoKHR::default()
                .connection(conn)
                .window(w.window.get());
            let f = khr::xcb_surface::Instance::new(entry, instance);
            unsafe { f.create_xcb_surface(&info, None) }.map_err(SrError::from)
        }
        (RawDisplayHandle::Android(_), RawWindowHandle::AndroidNdk(w)) => {
            let info = vk::AndroidSurfaceCreateInfoKHR::default()
                .window(w.a_native_window.as_ptr());
            let f = khr::android_surface::Instance::new(entry, instance);
            unsafe { f.create_android_surface(&info, None) }.map_err(SrError::from)
        }
        _ => Err(SrError::from(vk::Result::ERROR_EXTENSION_NOT_PRESENT)),
    }
}

/// Owns the `vk::SurfaceKHR` and the loader used to destroy it on drop.
///
/// Holds an `Rc<Core>` so the underlying `VkInstance` (owned by `Core`)
/// cannot be destroyed before this surface is. Without this clone, if the
/// `Surface` happens to be the last field dropped in its owner, the
/// swapchain / renderer may release their own `Rc<Core>` first, freeing the
/// instance and leaving `destroy_surface` pointing at a dead handle.
pub struct Surface {
    #[allow(dead_code)] // kept alive to extend the instance's lifetime
    core: Rc<vulkan_abstraction::Core>,
    instance_fn: khr::surface::Instance,
    handle: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(core: Rc<vulkan_abstraction::Core>, handle: vk::SurfaceKHR) -> Self {
        let instance_fn = khr::surface::Instance::new(core.entry(), core.instance());
        Self {
            core,
            instance_fn,
            handle,
        }
    }
    pub fn handle(&self) -> vk::SurfaceKHR {
        self.handle
    }
    pub fn instance_fn(&self) -> &khr::surface::Instance {
        &self.instance_fn
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        if self.handle != vk::SurfaceKHR::null() {
            unsafe { self.instance_fn.destroy_surface(self.handle, None) };
        }
    }
}
