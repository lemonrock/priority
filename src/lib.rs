// This file is part of priority. It is subject to the license terms in the COPYRIGHT file found in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/priority/master/COPYRIGHT. No part of priority, including this file, may be copied, modified, propagated, or distributed except according to the terms contained in the COPYRIGHT file.
// Copyright © 2016 The developers of priority. See the COPYRIGHT file in the top-level directory of this distribution and at https://raw.githubusercontent.com/lemonrock/priority/master/COPYRIGHT.

#![feature(associated_consts)]


extern crate libc;
use libc::c_int;
extern crate errno;
use errno::Errno;
use errno::errno;
use errno::set_errno;
use std::result::Result;
use std::io::ErrorKind;

#[cfg(not(target_os = "solaris"))] #[allow(non_camel_case_types)] pub type id_t = libc::id_t;
#[cfg(target_os = "solaris")] #[allow(non_camel_case_types)] pub type id_t = c_int;

#[derive(Debug, Copy, Clone)]
#[repr(i32)] // We'd like to use c_int here, but the compiler won't let us
#[cfg(not(target_os = "windows"))]
pub enum WhichPriority
{
	Process = libc::PRIO_PROCESS,
	ProcessGroup = libc::PRIO_PGRP,
	User = libc::PRIO_USER,
}

#[cfg(not(target_os = "windows"))]
impl WhichPriority
{
	pub const CurrentProcessOrThread: id_t = 0;
	pub const Minimum: i8 = libc::PRIO_MIN as i8;
	pub const Maximum: i8 = libc::PRIO_MAX as i8;
	
	/// May error with NotFound
	pub fn get(self, who: id_t) -> Result<i8, ErrorKind>
	{
		get_internal(self as c_int, who)
	}
	
	/// May error with NotFound or PermissionDenied
	pub fn set(self, who: id_t, priority: i8) -> Result<(), ErrorKind>
	{
		debug_assert!(priority >= WhichPriority::Minimum, "priority {} exceeds Minimum {}", priority, WhichPriority::Minimum);
		debug_assert!(priority <= WhichPriority::Maximum, "priority {} exceeds Maximum {}", priority, WhichPriority::Maximum);
	
		set_internal(self as c_int, who, priority as i32)
	}
}
	
fn get_internal(which: c_int, who: id_t) -> Result<i8, ErrorKind>
{
	set_errno(Errno(0));
	let priority = unsafe { libc::getpriority(which, who) } as i8;
	match errno().0
	{
		0 => Ok(priority),
		libc::EINVAL => panic!("Invalid value of which or who"),
		libc::ESRCH => Err(ErrorKind::NotFound),
		unexpected @ _ => panic!("Unexpected errno {}", unexpected),
	}
}

fn set_internal(which: c_int, who: id_t, priority: c_int) -> Result<(), ErrorKind>
{
	let result = unsafe { libc::setpriority(which, who, priority) };
	if result == 0
	{
		return Ok(())
	}

	match errno().0
	{
		libc::EINVAL => panic!("Invalid value of which or who"),
		libc::ESRCH => Err(ErrorKind::NotFound),
		libc::EACCES => Err(ErrorKind::PermissionDenied),
		libc::EPERM => Err(ErrorKind::PermissionDenied),
		unexpected @ _ => panic!("Unexpected errno {}", unexpected),
	}
}

#[cfg(target_os = "macos")]
pub fn put_current_process_into_background() -> Result<(), ErrorKind>
{
	put_current_x_into_background(libc::PRIO_DARWIN_PROCESS)
}

#[cfg(target_os = "macos")]
pub fn put_current_thread_into_background() -> Result<(), ErrorKind>
{
	put_current_x_into_background(libc::PRIO_DARWIN_THREAD)
}

#[cfg(target_os = "macos")]
pub fn get_current_process_out_of_background() -> Result<(), ErrorKind>
{
	get_current_x_out_of_background(libc::PRIO_DARWIN_PROCESS)
}

#[cfg(target_os = "macos")]
pub fn get_current_thread_out_of_background() -> Result<(), ErrorKind>
{
	get_current_x_out_of_background(libc::PRIO_DARWIN_THREAD)
}

#[cfg(target_os = "macos")]
pub fn is_current_process_in_background() -> Result<bool, ErrorKind>
{
	is_current_x_in_background(libc::PRIO_DARWIN_PROCESS)
}

#[cfg(target_os = "macos")]
pub fn is_current_thread_in_background() -> Result<bool, ErrorKind>
{
	is_current_x_in_background(libc::PRIO_DARWIN_THREAD)
}

#[cfg(target_os = "macos")]
fn put_current_x_into_background(which: c_int) -> Result<(), ErrorKind>
{
	set_internal(which, WhichPriority::CurrentProcessOrThread, libc::PRIO_DARWIN_BG)
}

#[cfg(target_os = "macos")]
fn get_current_x_out_of_background(which: c_int) -> Result<(), ErrorKind>
{
	set_internal(which, WhichPriority::CurrentProcessOrThread, 0)
}

#[cfg(target_os = "macos")]
fn is_current_x_in_background(which: c_int) -> Result<bool, ErrorKind>
{
	match try!(get_internal(which, WhichPriority::CurrentProcessOrThread))
	{
		0 => Ok(false),
		1 => Ok(true),
		unexpected @ _ => panic!("Did not expect priority {}", unexpected),
	}
}