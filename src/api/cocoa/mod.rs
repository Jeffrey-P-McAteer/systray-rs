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

use objc_id;
use objc_id::Id;

use objc_foundation;
use cocoa::foundation::{NSAutoreleasePool, NSString};
use objc_foundation::{INSObject, NSObject};

pub struct Window {
    name: String,
    menu: *mut objc::runtime::Object,
    pool: *mut objc::runtime::Object,
    event_tx: Sender<SystrayEvent>
}

impl Window {
    pub fn new(event_tx: Sender<SystrayEvent>) -> Result<Window, Error> {
        Ok(Window {
            name: String::new(),
            menu: unsafe { NSMenu::new(nil).autorelease() },
            pool: unsafe { NSAutoreleasePool::new(nil) },
            event_tx: event_tx,
        })
    }
    pub fn quit(&self) {
        unimplemented!()
    }
    pub fn set_tooltip(&self, _: &str) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn add_menu_entry(&self, _item_idx: u32, _item_name: &str) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn add_menu_separator(&mut self, _item_idx: u32) -> Result<u32, Error> {
        unimplemented!()
    }
    pub fn wait_for_message(&mut self) {
        unimplemented!()
    }
    pub fn set_icon_from_buffer(&self, _: &[u8], _: u32, _: u32) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn set_icon_from_resource(&self, _: &str) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn set_icon_from_file(&self, file: &str) -> Result<(), Error> {
        unimplemented!()
    }
    pub fn shutdown(&self) -> Result<(), Error> {
        unimplemented!()
    }
}
