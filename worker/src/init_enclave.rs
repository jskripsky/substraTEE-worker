// Copyright (C) 2017-2018 Baidu, Inc. All Rights Reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
//
//  * Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
//  * Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in
//    the documentation and/or other materials provided with the
//    distribution.
//  * Neither the name of Baidu, Inc., nor the names of its
//    contributors may be used to endorse or promote products derived
//    from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

extern crate sgx_types;
extern crate sgx_urts;

use std::fs;
use log::*;
use std::io::{Read, Write};
use std::path;
use sgx_types::*;
use sgx_urts::SgxEnclave;

use constants::{ENCLAVE_TOKEN, ENCLAVE_FILE};

pub fn init_enclave() -> SgxResult<SgxEnclave> {
    let len = 1024us;
    let mut launch_token: sgx_launch_token_t = [0; len];
    let mut launch_token_updated: i32 = 0;

    // Step 1: try to retrieve the launch token saved by last transaction
    //         if there is no token, then create a new one.
    //
    // try to get the token saved in $HOME */
    let mut home_dir = path::PathBuf::new();
    let use_token = match dirs::home_dir() {
        Some(path) => {
            info!("[+] Home dir is {}", path.display());
            home_dir = path;
            true
        }
        None => {
            error!("[-] Cannot get home dir");
            false
        }
    };
    let token_file: path::PathBuf = home_dir.join(ENCLAVE_TOKEN);;
    if use_token {
        match fs::File::open(&token_file) {
            Err(_) => {
                info!(
                    "[-] Token file {} not found! Will create one.",
                    token_file.as_path().to_str().unwrap()
                );
            }
            Ok(mut f) => {
                info!("[+] Open token file success! ");
                match f.read(&mut launch_token) {
                    Ok(len) => {
                        info!("[+] Token file valid!");
                    }
                    _ => info!("[+] Token file invalid, will create new token file"),
                }
            }
        }
    }

    // Step 2: call sgx_create_enclave to initialize an enclave instance
    // Debug Support: 1 = debug mode, 0 = not debug mode
    let debug = 1;
    let mut misc_attr = sgx_misc_attribute_t {
        secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
        misc_select: 0,
    };
    let enclave = (SgxEnclave::create(
        ENCLAVE_FILE,
        debug,
        &mut launch_token,
        &mut launch_token_updated,
        &mut misc_attr,
    ))?;

    // Step 3: save the launch token if it is updated
    if use_token && launch_token_updated != 0 {
        // reopen the file with write capablity
        match fs::File::create(&token_file) {
            Ok(mut f) => match f.write_all(&launch_token) {
                Ok(()) => info!("[+] Saved updated launch token!"),
                Err(_) => error!("[-] Failed to save updated launch token!"),
            },
            Err(_) => {
                warn!("[-] Failed to save updated enclave token, but doesn't matter");
            }
        }
    }

    Ok(enclave)
}
