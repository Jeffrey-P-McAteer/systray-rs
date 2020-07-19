#![allow(non_snake_case)]

use crate::{Error, SystrayEvent};
use std;

use std::sync::mpsc::{channel, Sender};

use std::mem;
use objc::Message;

use cocoa;
use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivateIgnoringOtherApps, NSMenu, NSMenuItem,
    NSRunningApplication, NSStatusBar, NSStatusItem, NSWindow,
};
use cocoa::base::{nil, YES /* id, class, BOOL */};

use libc;
use libc::c_void;
use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel};
use objc::{sel,sel_impl,msg_send};

use objc_id;
use objc_id::Id;

use objc_foundation;
use cocoa::foundation::{NSAutoreleasePool, NSString};
use objc_foundation::{INSObject, NSObject};

pub struct Window {
    name: String,
    menu: *mut objc::runtime::Object,
    pool: *mut objc::runtime::Object,
    app: *mut objc::runtime::Object,
    event_tx: Sender<SystrayEvent>
}

impl Window {
    pub fn new(event_tx: Sender<SystrayEvent>) -> Result<Window, Error> {
        let mut w = Window {
            name: String::new(),
            menu: unsafe { NSMenu::new(nil).autorelease() },
            pool: unsafe { NSAutoreleasePool::new(nil) },
            app: unsafe { NSApp() },
            event_tx: event_tx,
        };

        unsafe {
            w.app.activateIgnoringOtherApps_(YES);
            let item = NSStatusBar::systemStatusBar(nil).statusItemWithLength_(-1.0);
            let title = NSString::alloc(nil).init_str(&w.name);
            item.setTitle_(title);
            item.setMenu_(w.menu);

            let current_app = NSRunningApplication::currentApplication(nil);
            current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);

            // I think this blocks
            w.app.run();
            println!("After app.run()");
        }

        Ok(w)
    }
    pub fn quit(&self) {
        unsafe {
            let terminate_fn = (*self.app).class().instance_method(sel!(terminate:))
                .expect("No method terminate: found")
                .implementation();
            terminate_fn();
        }
    }
    pub fn set_tooltip(&self, _: &str) -> Result<(), Error> {
        Err(Error::NotImplementedError)
    }
    pub fn add_menu_entry(&self, _item_idx: u32, item_name: &str) -> Result<(), Error> {
        unsafe {
            let cb_obj = Callback::from(Box::new(|| {
                println!("cb_obj ran from add_menu_entry");
            }));

            let no_key = NSString::alloc(nil).init_str(""); // TODO want this eventually

            let itemtitle = NSString::alloc(nil).init_str(item_name);
            let action = sel!(call);
            let item = NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(itemtitle, action, no_key);
            // Type inferance fails here, but we don't really
            // care about the return values so assigning
            // to _ with a () type annotation fixes a compile
            // time error
            let _: () = msg_send![item, setTarget: cb_obj];

            NSMenu::addItem_(self.menu, item);
        }
        Ok(())
    }
    pub fn add_menu_separator(&mut self, _item_idx: u32) -> Result<u32, Error> {
        Err(Error::NotImplementedError)
    }
    pub fn set_icon_from_buffer(&self, _: &[u8], _: u32, _: u32) -> Result<(), Error> {
        Err(Error::NotImplementedError)
    }
    pub fn set_icon_from_resource(&self, _: &str) -> Result<(), Error> {
        Err(Error::NotImplementedError)
    }
    pub fn set_icon_from_file(&self, file: &str) -> Result<(), Error> {
        Err(Error::NotImplementedError)
    }
    pub fn shutdown(&self) -> Result<(), Error> {
        self.quit();
        Ok(())
    }
}


// this code is pretty much a rip off of
// https://github.com/SSheldon/rust-objc-foundation/blob/master/examples/custom_class.rs

enum Callback {}
unsafe impl Message for Callback {}

// SO.. some explanation is in order here.  We want to allow closure callbacks that
// can modify their environment.  But we can't keep them on the $name object because
// that is really just a stateless proxy for the objc object.  So we store them
// as numeric pointers values in "ivar" fields on that object.  But, if we store a pointer to the
// closure object, we'll run into issues with thin/fat pointer conversions (because
// closure objects are trait objects and thus fat pointers).  So we wrap the closure in
// another boxed object ($cbs_name), which, since it doesn't use traits, is actually a
// regular "thin" pointer, and store THAT pointer in the ivar.  But...so...oy.
struct CallbackState {
    cb: Box<dyn Fn() -> ()>,
}

impl Callback {
    fn from(cb: Box<dyn Fn() -> ()>) -> Id<Self> {
        let cbs = CallbackState { cb };
        let bcbs = Box::new(cbs);

        let ptr = Box::into_raw(bcbs);
        let ptr = ptr as *mut c_void as usize;
        let mut oid = <Callback as INSObject>::new();
        (*oid).setptr(ptr);
        oid
    }

    fn setptr(&mut self, uptr: usize) {
        unsafe {
            let obj = &mut *(self as *mut _ as *mut ::objc::runtime::Object);
            obj.set_ivar("_cbptr", uptr);
        }
    }
}

// TODO: Drop for $name doesn't get called, probably because objc manages the memory and
// releases it for us.  so we leak the boxed callback right now.

impl INSObject for Callback {
    fn class() -> &'static Class {
        let cname = "Callback";

        let mut klass = Class::get(cname);
        if klass.is_none() {
            let superclass = NSObject::class();
            let mut decl = ClassDecl::new(&cname, superclass).unwrap();
            decl.add_ivar::<usize>("_cbptr");

            extern "C" fn sysbar_callback_call(this: &Object, _cmd: Sel) {
                unsafe {
                    let pval: usize = *this.get_ivar("_cbptr");
                    let ptr = pval as *mut c_void;
                    let ptr = ptr as *mut CallbackState;
                    let bcbs: Box<CallbackState> = Box::from_raw(ptr);
                    {
                        (*bcbs.cb)();
                    }
                    mem::forget(bcbs);
                }
            }

            unsafe {
                decl.add_method(
                    sel!(call),
                    sysbar_callback_call as extern "C" fn(&Object, Sel),
                );
            }

            decl.register();
            klass = Class::get(cname);
        }
        klass.unwrap()
    }
}

