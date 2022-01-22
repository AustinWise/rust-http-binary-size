use std::ffi::c_void;
use std::ffi::OsString;
use std::io;
use std::ptr;

use windows::Win32::Foundation::PWSTR;
use windows::Win32::Networking::WinHttp::*;

struct MyWinHttp(*mut ::core::ffi::c_void);

impl MyWinHttp {
    fn new() -> Result<Self, io::Error> {
        unsafe {
            let http_sess = WinHttpOpen(
                "Austin Wise",
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, // NOTE: only supported on Windows 8.1 and higher
                PWSTR::default(),
                PWSTR::default(),
                0,
            );
            if http_sess == ptr::null_mut() {
                Err(io::Error::last_os_error())
            } else {
                Ok(MyWinHttp(http_sess))
            }
        }
    }

    fn connect(&self, server_name: &str, port: u16) -> Result<MyConnect, io::Error> {
        unsafe {
            let con = WinHttpConnect(self.0, server_name, port.into(), 0);
            if con == ptr::null_mut() {
                Err(io::Error::last_os_error())
            } else {
                Ok(MyConnect(con, self))
            }
        }
    }
}

impl Drop for MyWinHttp {
    fn drop(&mut self) {
        unsafe {
            assert!(WinHttpCloseHandle(self.0).as_bool());
        }
    }
}

struct MyConnect<'a>(*mut ::core::ffi::c_void, &'a MyWinHttp);

impl<'a> MyConnect<'a> {
    fn open_request_get(&self, path: &str) -> Result<MyRequest, io::Error> {
        unsafe {
            let mut accept_types = PWSTR::default();
            let req = WinHttpOpenRequest(
                self.0,
                PWSTR::default(),
                path,
                PWSTR::default(),
                PWSTR::default(),
                &mut accept_types,
                0,
            );
            if req == ptr::null_mut() {
                Err(io::Error::last_os_error())
            } else {
                Ok(MyRequest(req, self))
            }
        }
    }
}

impl<'a> Drop for MyConnect<'a> {
    fn drop(&mut self) {
        unsafe {
            assert!(WinHttpCloseHandle(self.0).as_bool());
        }
    }
}

struct MyRequest<'a>(*mut ::core::ffi::c_void, &'a MyConnect<'a>);

impl<'a> Drop for MyRequest<'a> {
    fn drop(&mut self) {
        unsafe {
            assert!(WinHttpCloseHandle(self.0).as_bool());
        }
    }
}

//TODO: combine the different methods into one that consumes the HINTERNET,
//so that functions cannot be called in the wrong order.
impl<'a> MyRequest<'a> {
    fn send(&self) -> Result<(), io::Error> {
        unsafe {
            if WinHttpSendRequest(self.0, PWSTR::default(), 0, ptr::null(), 0, 0, 0).as_bool() {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    fn receive_response(&self) -> Result<(), io::Error> {
        unsafe {
            if WinHttpReceiveResponse(self.0, ptr::null_mut()).as_bool() {
                Ok(())
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    fn query_status(&self) -> Result<u32, io::Error> {
        unsafe {
            let mut code: u32 = 0;
            let mut buf_length: u32 = std::mem::size_of_val(&code).try_into().unwrap();
            let res = WinHttpQueryHeaders(
                self.0,
                WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                PWSTR::default(),
                &mut code as *mut u32 as *mut c_void,
                &mut buf_length as *mut u32,
                ptr::null_mut(),
            );
            if res.as_bool() {
                Ok(code)
            } else {
                Err(io::Error::last_os_error())
            }
        }
    }

    fn print_all_data(&self) -> Result<(), io::Error> {
        unsafe {
            let mut buf: Vec<u8> = vec![0; 4096];
            let mut bytes_read: u32 = 0;
            while WinHttpQueryDataAvailable(self.0, &mut bytes_read as *mut u32).as_bool()
                && bytes_read != 0
            {
                println!("data available");
                if WinHttpReadData(
                    self.0,
                    buf.as_mut_ptr() as *mut c_void,
                    bytes_read,
                    &mut bytes_read as *mut u32,
                )
                .as_bool()
                    && bytes_read != 0
                {
                    println!("got some data: {} {}", bytes_read, buf.len());
                    let bytes_read: usize = bytes_read.try_into().unwrap();
                    println!("{}", std::str::from_utf8(&buf[..bytes_read]).unwrap());
                } else {
                    return Err(io::Error::last_os_error());
                }
            }
            Ok(())
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let http = MyWinHttp::new()?;
    let con = http.connect("127.0.0.1", 8000)?;
    let req = con.open_request_get("/page/README.md")?;
    req.send()?;
    req.receive_response()?;
    println!("status code: {}", req.query_status()?);
    req.print_all_data()?;
    Ok(())
}
